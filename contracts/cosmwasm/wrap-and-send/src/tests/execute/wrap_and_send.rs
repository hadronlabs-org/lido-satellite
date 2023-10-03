use crate::{
    contract::execute,
    msg::ExecuteMsg,
    tests::helpers::{bin_request_to_query_request, craft_wrap_and_send_msg, mock_instantiate},
};
use astroport::router::{
    ExecuteMsg::ExecuteSwapOperations as AstroportExecuteSwapOperations, SwapOperation,
};
use cosmwasm_std::{
    coin, coins,
    testing::{mock_info, MockQuerier, MOCK_CONTRACT_ADDR},
    to_binary, ContractResult, CosmosMsg, Querier, QuerierResult, QueryRequest, SystemResult,
    Uint128, WasmMsg,
};
use lido_satellite::msg::ExecuteMsg::Mint as LidoSatelliteExecuteMint;
use neutron_sdk::{
    bindings::{msg::IbcFee, query::NeutronQuery},
    query::min_ibc_fee::MinIbcFeeResponse,
};

// in these tests, we don't care about the actual error,
// we only care that contact returns an error and reverts execution
mod invalid_funds {
    use super::*;

    #[test]
    fn no_funds() {
        let (mut deps, env) = mock_instantiate::<MockQuerier>();
        execute(
            deps.as_mut(),
            env,
            mock_info("stranger", &[]),
            craft_wrap_and_send_msg(0u128),
        )
        .unwrap_err();
    }

    #[test]
    fn wrong_denom() {
        let (mut deps, env) = mock_instantiate::<MockQuerier>();
        execute(
            deps.as_mut(),
            env,
            mock_info("stranger", &[coin(200, "denom1")]),
            craft_wrap_and_send_msg(0u128),
        )
        .unwrap_err();
    }

    #[test]
    fn all_wrong_denoms() {
        let (mut deps, env) = mock_instantiate::<MockQuerier>();
        execute(
            deps.as_mut(),
            env,
            mock_info("stranger", &[coin(200, "denom1"), coin(300, "denom2")]),
            craft_wrap_and_send_msg(0u128),
        )
        .unwrap_err();
    }

    #[test]
    fn extra_denoms() {
        let (mut deps, env) = mock_instantiate::<MockQuerier>();
        execute(
            deps.as_mut(),
            env,
            mock_info(
                "stranger",
                &[coin(200, "bridged_denom"), coin(300, "denom2")],
            ),
            craft_wrap_and_send_msg(0u128),
        )
        .unwrap_err();
    }

    #[test]
    fn not_enough_for_ibc_fee() {
        let (mut deps, env) = mock_instantiate::<MockQuerier>();
        execute(
            deps.as_mut(),
            env,
            mock_info("stranger", &[coin(200, "bridged_denom")]),
            craft_wrap_and_send_msg(300u128),
        )
        .unwrap_err();
    }
}

#[derive(Default)]
struct CustomMockQuerier {}

impl Querier for CustomMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request = match bin_request_to_query_request::<NeutronQuery>(bin_request) {
            Ok(v) => v,
            Err(e) => return e,
        };
        match request {
            QueryRequest::Custom(query) => match query {
                NeutronQuery::MinIbcFee {} => {
                    SystemResult::Ok(ContractResult::from(to_binary(&MinIbcFeeResponse {
                        min_fee: IbcFee {
                            recv_fee: vec![],
                            ack_fee: vec![coin(20, "ibc_fee_denom"), coin(45, "untrn")],
                            timeout_fee: vec![coin(30, "ibc_fee_denom"), coin(55, "untrn")],
                        },
                    })))
                }
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }
}

#[test]
fn success() {
    let (mut deps, env) = mock_instantiate::<CustomMockQuerier>();
    let response = execute(
        deps.as_mut(),
        env,
        mock_info("stranger", &[coin(300, "bridged_denom")]),
        ExecuteMsg::WrapAndSend {
            source_port: "source_port".to_string(),
            source_channel: "source_channel".to_string(),
            receiver: "receiver".to_string(),
            amount_to_swap_for_ibc_fee: Uint128::new(100),
            ibc_fee_denom: "ibc_fee_denom".to_string(),
            astroport_swap_operations: vec![SwapOperation::NativeSwap {
                offer_denom: "canonical_denom".to_string(),
                ask_denom: "ibc_fee_denom".to_string(),
            }],
            refund_address: "refund_address".to_string(),
        },
    )
    .unwrap();
    assert_eq!(response.messages.len(), 3);
    assert_eq!(
        response.messages[0].msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "lido_satellite".to_string(),
            msg: to_binary(&LidoSatelliteExecuteMint { receiver: None }).unwrap(),
            funds: coins(300, "bridged_denom"),
        })
    );
    assert_eq!(
        response.messages[1].msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "astroport_router".to_string(),
            msg: to_binary(&AstroportExecuteSwapOperations {
                operations: vec![SwapOperation::NativeSwap {
                    offer_denom: "canonical_denom".to_string(),
                    ask_denom: "ibc_fee_denom".to_string()
                }],
                minimum_receive: Some(Uint128::new(50)),
                to: None,
                max_spread: None,
            })
            .unwrap(),
            funds: coins(100, "canonical_denom"),
        })
    );
    assert_eq!(
        response.messages[2].msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_CONTRACT_ADDR.to_string(),
            msg: to_binary(&ExecuteMsg::SwapCallback {
                source_port: "source_port".to_string(),
                source_channel: "source_channel".to_string(),
                receiver: "receiver".to_string(),
                amount_to_send: coin(200, "canonical_denom"),
                min_ibc_fee: IbcFee {
                    recv_fee: vec![],
                    ack_fee: coins(20, "ibc_fee_denom"),
                    timeout_fee: coins(30, "ibc_fee_denom"),
                },
                refund_address: "refund_address".to_string(),
            })
            .unwrap(),
            funds: vec![],
        })
    );
    // TODO: check attributes as well
}
