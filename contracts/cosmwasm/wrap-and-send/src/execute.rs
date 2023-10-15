use crate::{
    contract::{IBC_TRANSFER_REPLY_ID, WRAP_CALLBACK_REPLY_ID},
    msg::ExecuteMsg,
    state::{
        IbcTransferInfo, RefundInfo, CONFIG, EXECUTION_FLAG, IBC_TRANSFER_CONTEXT, REFUND_INFO,
    },
    ContractError, ContractResult,
};
use astroport::router::{
    ExecuteMsg::ExecuteSwapOperations as AstroportExecuteSwapOperations, SwapOperation,
};
use cosmwasm_std::{
    attr, coin, to_binary, Addr, BankMsg, Coin, Deps, DepsMut, Env, MessageInfo, Response, SubMsg,
    Uint128, WasmMsg,
};
use lido_satellite::{
    error::ContractError as LidoSatelliteContractError, execute::find_denom,
    msg::ExecuteMsg::Mint as LidoSatelliteExecuteMint,
};
use neutron_sdk::{
    bindings::{
        msg::{IbcFee, NeutronMsg},
        query::NeutronQuery,
    },
    query::min_ibc_fee::query_min_ibc_fee,
    sudo::msg::RequestPacketTimeoutHeight,
};
use std::cmp::Ordering;

#[allow(clippy::too_many_arguments)]
pub(crate) fn execute_wrap_and_send(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    source_port: String,
    source_channel: String,
    receiver: String,
    amount_to_swap_for_ibc_fee: Uint128,
    ibc_fee_denom: String,
    astroport_swap_operations: Vec<SwapOperation>,
    refund_address: String,
) -> ContractResult<Response<NeutronMsg>> {
    if let Some(true) = EXECUTION_FLAG.may_load(deps.storage)? {
        return Err(ContractError::AlreadyInExecution {});
    }
    EXECUTION_FLAG.save(deps.storage, &true)?;

    let config = CONFIG.load(deps.storage)?;
    let mut response = Response::new().add_attribute("action", "wrap_and_send");

    let received_amount = find_denom(&info.funds, &config.bridged_denom)?
        .ok_or(LidoSatelliteContractError::NothingToMint {})?
        .amount;
    response = response.add_attribute(
        "received_amount",
        format!("{}{}", received_amount, config.bridged_denom),
    );

    let refund_address = deps.api.addr_validate(&refund_address)?;
    response = response.add_attribute("refund_address", &refund_address);

    let potential_refund = coin(received_amount.u128(), config.canonical_denom);
    REFUND_INFO.save(
        deps.storage,
        &RefundInfo {
            refund_address: refund_address.clone(),
            funds: potential_refund,
        },
    )?;

    let wrap_msg = WasmMsg::Execute {
        contract_addr: config.lido_satellite.into_string(),
        msg: to_binary(&LidoSatelliteExecuteMint { receiver: None })?,
        funds: info.funds,
    };
    let callback_msg = WasmMsg::Execute {
        contract_addr: env.contract.address.into_string(),
        msg: to_binary(&ExecuteMsg::WrapCallback {
            source_port,
            source_channel,
            receiver,
            amount_to_swap_for_ibc_fee,
            ibc_fee_denom,
            astroport_swap_operations,
            received_amount,
            refund_address,
        })?,
        funds: vec![],
    };

    Ok(response
        .add_message(wrap_msg)
        .add_submessage(SubMsg::reply_on_error(callback_msg, WRAP_CALLBACK_REPLY_ID)))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn execute_wrap_callback(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    source_port: String,
    source_channel: String,
    receiver: String,
    amount_to_swap_for_ibc_fee: Uint128,
    ibc_fee_denom: String,
    astroport_swap_operations: Vec<SwapOperation>,
    received_amount: Uint128,
    refund_address: Addr,
) -> ContractResult<Response<NeutronMsg>> {
    if info.sender != env.contract.address {
        return Err(ContractError::InternalMethod {});
    }

    if amount_to_swap_for_ibc_fee.is_zero() {
        return Err(ContractError::ZeroForSwap {});
    } else if amount_to_swap_for_ibc_fee >= received_amount {
        return Err(ContractError::NotEnoughForSwap {});
    }

    let config = CONFIG.load(deps.storage)?;
    let mut response = Response::new();

    let amount_to_swap_for_ibc_fee =
        coin(amount_to_swap_for_ibc_fee.u128(), &config.canonical_denom);
    response = response.add_attribute(
        "amount_to_swap_for_ibc_fee",
        amount_to_swap_for_ibc_fee.to_string(),
    );

    let amount_to_send = coin(
        (received_amount - amount_to_swap_for_ibc_fee.amount).u128(),
        &config.canonical_denom,
    );
    response = response.add_attribute("amount_to_send", amount_to_send.to_string());

    let min_ibc_fee = calculate_min_ibc_fee(deps.as_ref(), &ibc_fee_denom)?;
    response = response.add_attribute(
        "min_ibc_fee",
        format!(
            "{}{}",
            min_ibc_fee.ack_fee[0].amount + min_ibc_fee.timeout_fee[0].amount,
            min_ibc_fee.ack_fee[0].denom
        ),
    );

    let swap_msg = WasmMsg::Execute {
        contract_addr: config.astroport_router.into_string(),
        msg: to_binary(&AstroportExecuteSwapOperations {
            operations: astroport_swap_operations,
            minimum_receive: Some(
                min_ibc_fee.ack_fee[0].amount + min_ibc_fee.timeout_fee[0].amount,
            ),
            to: None,
            max_spread: None,
        })?,
        funds: vec![amount_to_swap_for_ibc_fee],
    };
    let callback_msg = WasmMsg::Execute {
        contract_addr: env.contract.address.into_string(),
        msg: to_binary(&ExecuteMsg::SwapCallback {
            source_port,
            source_channel,
            receiver,
            amount_to_send,
            min_ibc_fee,
            refund_address,
        })?,
        funds: vec![],
    };

    Ok(response.add_messages([swap_msg, callback_msg]))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn execute_swap_callback(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    source_port: String,
    source_channel: String,
    receiver: String,
    amount_to_send: Coin,
    min_ibc_fee: IbcFee,
    refund_address: Addr,
) -> ContractResult<Response<NeutronMsg>> {
    if info.sender != env.contract.address {
        return Err(ContractError::InternalMethod {});
    }

    let mut response = Response::new();

    let total_ibc_fee = min_ibc_fee.ack_fee[0].amount + min_ibc_fee.timeout_fee[0].amount;
    let ibc_fee_denom = &min_ibc_fee.ack_fee[0].denom;
    let fee_balance = deps
        .querier
        .query_balance(&env.contract.address, ibc_fee_denom)?;
    let refund = match fee_balance.amount.cmp(&total_ibc_fee) {
        Ordering::Less => {
            // should never happen, but let's be cautious
            return Err(ContractError::SwappedForLessThanRequested {});
        }
        Ordering::Equal => None,
        Ordering::Greater => Some(fee_balance.amount - total_ibc_fee),
    };

    if let Some(refund) = refund {
        let refund = coin(refund.u128(), ibc_fee_denom);
        response = response
            .add_attribute("extra_ibc_fee_refunded", refund.to_string())
            .add_message(BankMsg::Send {
                to_address: refund_address.to_string(),
                amount: vec![refund],
            })
    }

    IBC_TRANSFER_CONTEXT.save(
        deps.storage,
        &IbcTransferInfo {
            refund_address,
            ibc_fee: min_ibc_fee.clone(),
            sent_amount: amount_to_send.clone(),
        },
    )?;

    // 20 minutes should be enough for IBC transfer to go through
    // FIXME: maybe better allow user to set their own timeout?
    let timeout_timestamp = env.block.time.plus_minutes(20).nanos();
    response = response.add_attributes([
        attr("source_port", &source_port),
        attr("source_channel", &source_channel),
        attr("receiver", &receiver),
    ]);
    let ibc_transfer = NeutronMsg::IbcTransfer {
        source_port,
        source_channel,
        token: amount_to_send,
        sender: env.contract.address.into_string(),
        receiver,
        timeout_height: RequestPacketTimeoutHeight {
            revision_number: None,
            revision_height: None,
        },
        timeout_timestamp,
        memo: "".to_string(),
        fee: min_ibc_fee,
    };

    Ok(response.add_submessage(SubMsg::reply_on_success(
        ibc_transfer,
        IBC_TRANSFER_REPLY_ID,
    )))
}

fn calculate_min_ibc_fee(deps: Deps<NeutronQuery>, ibc_fee_denom: &str) -> ContractResult<IbcFee> {
    let mut fee = query_min_ibc_fee(deps)?.min_fee;
    fee.ack_fee.retain(|coin| coin.denom == ibc_fee_denom);
    fee.timeout_fee.retain(|coin| coin.denom == ibc_fee_denom);

    if !fee.recv_fee.is_empty() || fee.ack_fee.len() != 1 || fee.timeout_fee.len() != 1 {
        return Err(ContractError::MinIbcFee {});
    }
    if fee.ack_fee[0].amount.is_zero() || fee.timeout_fee[0].amount.is_zero() {
        return Err(ContractError::MinIbcFee {});
    }

    Ok(fee)
}

#[cfg(test)]
mod calculate_min_ibc_fee_tests {
    use super::calculate_min_ibc_fee;
    use crate::{
        tests::helpers::{bin_request_to_query_request, mock_instantiate},
        ContractError,
    };
    use cosmwasm_std::{
        coin, coins, to_binary, ContractResult, Querier, QuerierResult, QueryRequest, SystemResult,
    };
    use neutron_sdk::{
        bindings::{msg::IbcFee, query::NeutronQuery},
        query::min_ibc_fee::MinIbcFeeResponse,
    };
    use std::marker::PhantomData;

    trait IbcFeeResult {
        fn ibc_fee_result() -> IbcFee;
    }
    #[derive(Default)]
    struct CustomMockQuerier<T: IbcFeeResult> {
        ibc_fee_result: PhantomData<T>,
    }
    impl<T: IbcFeeResult> Querier for CustomMockQuerier<T> {
        fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
            let request = match bin_request_to_query_request::<NeutronQuery>(bin_request) {
                Ok(v) => v,
                Err(e) => return e,
            };
            match request {
                QueryRequest::Custom(query) => match query {
                    NeutronQuery::MinIbcFee {} => {
                        SystemResult::Ok(ContractResult::from(to_binary(&MinIbcFeeResponse {
                            min_fee: T::ibc_fee_result(),
                        })))
                    }
                    _ => unimplemented!(),
                },
                _ => unimplemented!(),
            }
        }
    }

    #[derive(Default)]
    struct NormalIbcFee {}
    impl IbcFeeResult for NormalIbcFee {
        fn ibc_fee_result() -> IbcFee {
            IbcFee {
                recv_fee: vec![],
                ack_fee: vec![coin(25, "ibc_fee_denom"), coin(45, "untrn")],
                timeout_fee: vec![coin(30, "ibc_fee_denom"), coin(55, "untrn")],
            }
        }
    }
    #[test]
    fn success() {
        let (deps, _env) = mock_instantiate::<CustomMockQuerier<NormalIbcFee>>();
        let ibc_fee = calculate_min_ibc_fee(deps.as_ref(), "ibc_fee_denom").unwrap();
        assert_eq!(
            ibc_fee,
            IbcFee {
                recv_fee: vec![],
                ack_fee: coins(25, "ibc_fee_denom"),
                timeout_fee: coins(30, "ibc_fee_denom"),
            }
        );
    }

    #[derive(Default)]
    struct RecvFeeSet {}
    impl IbcFeeResult for RecvFeeSet {
        fn ibc_fee_result() -> IbcFee {
            IbcFee {
                recv_fee: vec![coin(15, "ibc_fee_denom"), coin(35, "untrn")],
                ack_fee: vec![coin(25, "ibc_fee_denom"), coin(45, "untrn")],
                timeout_fee: vec![coin(30, "ibc_fee_denom"), coin(55, "untrn")],
            }
        }
    }
    #[test]
    fn recv_fee_is_not_empty() {
        let (deps, _env) = mock_instantiate::<CustomMockQuerier<RecvFeeSet>>();
        let err = calculate_min_ibc_fee(deps.as_ref(), "ibc_fee_denom").unwrap_err();
        assert_eq!(err, ContractError::MinIbcFee {});
    }

    #[derive(Default)]
    struct WeirdAckFee {}
    impl IbcFeeResult for WeirdAckFee {
        fn ibc_fee_result() -> IbcFee {
            IbcFee {
                recv_fee: vec![],
                ack_fee: vec![coin(25, "weird_denom1"), coin(45, "weird_denom2")],
                timeout_fee: vec![coin(30, "ibc_fee_denom"), coin(55, "untrn")],
            }
        }
    }
    #[test]
    fn ack_fee_has_no_required_denom() {
        let (deps, _env) = mock_instantiate::<CustomMockQuerier<WeirdAckFee>>();
        let err = calculate_min_ibc_fee(deps.as_ref(), "ibc_fee_denom").unwrap_err();
        assert_eq!(err, ContractError::MinIbcFee {});
    }

    #[derive(Default)]
    struct ZeroAckFee {}
    impl IbcFeeResult for ZeroAckFee {
        fn ibc_fee_result() -> IbcFee {
            IbcFee {
                recv_fee: vec![],
                ack_fee: vec![coin(0, "ibc_fee_denom"), coin(0, "untrn")],
                timeout_fee: vec![coin(30, "ibc_fee_denom"), coin(55, "untrn")],
            }
        }
    }
    #[test]
    fn zero_ack_fee() {
        let (deps, _env) = mock_instantiate::<CustomMockQuerier<ZeroAckFee>>();
        let err = calculate_min_ibc_fee(deps.as_ref(), "ibc_fee_denom").unwrap_err();
        assert_eq!(err, ContractError::MinIbcFee {});
    }

    #[derive(Default)]
    struct WeirdTimeoutFee {}
    impl IbcFeeResult for WeirdTimeoutFee {
        fn ibc_fee_result() -> IbcFee {
            IbcFee {
                recv_fee: vec![],
                ack_fee: vec![coin(25, "ibc_fee_denom"), coin(45, "untrn")],
                timeout_fee: vec![coin(30, "weird_denom1"), coin(55, "weird_denom2")],
            }
        }
    }
    #[test]
    fn timeout_fee_has_no_required_denom() {
        let (deps, _env) = mock_instantiate::<CustomMockQuerier<WeirdTimeoutFee>>();
        let err = calculate_min_ibc_fee(deps.as_ref(), "ibc_fee_denom").unwrap_err();
        assert_eq!(err, ContractError::MinIbcFee {});
    }

    #[derive(Default)]
    struct ZeroTimeoutFee {}
    impl IbcFeeResult for ZeroTimeoutFee {
        fn ibc_fee_result() -> IbcFee {
            IbcFee {
                recv_fee: vec![],
                ack_fee: vec![coin(25, "ibc_fee_denom"), coin(45, "untrn")],
                timeout_fee: vec![coin(0, "ibc_fee_denom"), coin(0, "untrn")],
            }
        }
    }
    #[test]
    fn zero_timeout_fee() {
        let (deps, _env) = mock_instantiate::<CustomMockQuerier<ZeroTimeoutFee>>();
        let err = calculate_min_ibc_fee(deps.as_ref(), "ibc_fee_denom").unwrap_err();
        assert_eq!(err, ContractError::MinIbcFee {});
    }
}
