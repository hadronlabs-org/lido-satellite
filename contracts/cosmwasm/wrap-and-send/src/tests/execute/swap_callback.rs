use crate::{
    contract::execute,
    state::{IbcTransferInfo, IBC_TRANSFER_CONTEXT},
    tests::helpers::{bin_request_to_query_request, craft_swap_callback_msg, mock_instantiate},
    ContractError,
};
use cosmwasm_std::{
    attr, coin, coins,
    testing::{mock_info, MockQuerier, MOCK_CONTRACT_ADDR},
    to_binary, Addr, BalanceResponse, BankMsg, BankQuery, ContractResult, CosmosMsg, Deps, Empty,
    Env, Querier, QuerierResult, QueryRequest, SystemResult,
};
use neutron_sdk::{
    bindings::{
        msg::{IbcFee, NeutronMsg},
        query::NeutronQuery,
    },
    sudo::msg::RequestPacketTimeoutHeight,
};
use std::marker::PhantomData;

trait BankBalanceResult {
    fn bank_balance_result() -> u128;
}
#[derive(Default)]
struct CustomMockQuerier<T: BankBalanceResult> {
    bank_balance_result: PhantomData<T>,
}
impl<T: BankBalanceResult> Querier for CustomMockQuerier<T> {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request = match bin_request_to_query_request::<Empty>(bin_request) {
            Ok(v) => v,
            Err(e) => return e,
        };
        match request {
            QueryRequest::Bank(query) => match query {
                // we want to make sure that contract only queries for bank balance
                BankQuery::Balance { address, denom } => {
                    // we want to make sure that contract only queries balance of itself
                    assert_eq!(address, MOCK_CONTRACT_ADDR);
                    // we want to make sure that contract only queries for IBC fee denom balance
                    assert_eq!(denom, "ibc_fee_denom");
                    SystemResult::Ok(ContractResult::from(to_binary(&BalanceResponse {
                        amount: coin(T::bank_balance_result(), "ibc_fee_denom"),
                    })))
                }
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }
}

#[test]
fn private_method() {
    let (mut deps, env) = mock_instantiate::<MockQuerier>();
    let err = execute(
        deps.as_mut(),
        env,
        mock_info("stranger", &[]),
        craft_swap_callback_msg(),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InternalMethod {});
}

#[derive(Default)]
struct LessThanRequired {}
impl BankBalanceResult for LessThanRequired {
    fn bank_balance_result() -> u128 {
        20
    }
}
#[test]
fn swapped_for_less_than_required() {
    let (mut deps, env) = mock_instantiate::<CustomMockQuerier<LessThanRequired>>();
    let err = execute(
        deps.as_mut(),
        env,
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        craft_swap_callback_msg(),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::SwappedForLessThanRequested {});
}

#[derive(Default)]
struct ExactlyAsRequired {}
impl BankBalanceResult for ExactlyAsRequired {
    fn bank_balance_result() -> u128 {
        50
    }
}
#[test]
fn swapped_for_exact_amount() {
    let (mut deps, env) = mock_instantiate::<CustomMockQuerier<ExactlyAsRequired>>();
    let response = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        craft_swap_callback_msg(),
    )
    .unwrap();

    assert_ibc_transfer_context(deps.as_ref());

    assert_eq!(response.messages.len(), 1);
    assert_eq!(response.messages[0].msg, craft_ibc_transfer_message(&env));

    assert_eq!(
        response.attributes,
        vec![
            attr("source_port", "source_port"),
            attr("source_channel", "source_channel"),
            attr("receiver", "receiver"),
            attr(
                "timeout_timestamp",
                env.block.time.plus_nanos(1000).nanos().to_string()
            ),
            attr("ibc_memo", "memo"),
        ]
    );
}

#[derive(Default)]
struct MoreThanRequired {}
impl BankBalanceResult for MoreThanRequired {
    fn bank_balance_result() -> u128 {
        60
    }
}
#[test]
fn swapped_for_more_than_required() {
    let (mut deps, env) = mock_instantiate::<CustomMockQuerier<MoreThanRequired>>();
    let response = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        craft_swap_callback_msg(),
    )
    .unwrap();

    assert_ibc_transfer_context(deps.as_ref());

    assert_eq!(response.messages.len(), 2);
    assert_eq!(
        response.messages[0].msg,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "refund_address".to_string(),
            amount: coins(10, "ibc_fee_denom")
        })
    );
    assert_eq!(response.messages[1].msg, craft_ibc_transfer_message(&env));

    assert_eq!(
        response.attributes,
        vec![
            attr("extra_ibc_fee_refunded", "10ibc_fee_denom"),
            attr("source_port", "source_port"),
            attr("source_channel", "source_channel"),
            attr("receiver", "receiver"),
            attr(
                "timeout_timestamp",
                env.block.time.plus_nanos(1000).nanos().to_string()
            ),
            attr("ibc_memo", "memo"),
        ]
    );
}

fn assert_ibc_transfer_context(deps: Deps<NeutronQuery>) {
    let ibc_transfer_context = IBC_TRANSFER_CONTEXT.load(deps.storage).unwrap();
    assert_eq!(
        ibc_transfer_context,
        IbcTransferInfo {
            refund_address: Addr::unchecked("refund_address"),
            ibc_fee: IbcFee {
                recv_fee: vec![],
                ack_fee: coins(20, "ibc_fee_denom"),
                timeout_fee: coins(30, "ibc_fee_denom"),
            },
            sent_amount: coin(200, "canonical_denom"),
        }
    );
}

fn craft_ibc_transfer_message(env: &Env) -> CosmosMsg<NeutronMsg> {
    CosmosMsg::Custom(NeutronMsg::IbcTransfer {
        source_port: "source_port".to_string(),
        source_channel: "source_channel".to_string(),
        token: coin(200, "canonical_denom"),
        sender: MOCK_CONTRACT_ADDR.to_string(),
        receiver: "receiver".to_string(),
        timeout_height: RequestPacketTimeoutHeight {
            revision_number: None,
            revision_height: None,
        },
        timeout_timestamp: env.block.time.plus_nanos(1000).nanos(),
        memo: "memo".to_string(),
        fee: IbcFee {
            recv_fee: vec![],
            ack_fee: coins(20, "ibc_fee_denom"),
            timeout_fee: coins(30, "ibc_fee_denom"),
        },
    })
}
