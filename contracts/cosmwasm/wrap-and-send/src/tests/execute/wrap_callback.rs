use crate::{
    contract::execute,
    tests::helpers::{
        bin_request_to_query_request, craft_swap_callback_msg, craft_wrap_callback_msg,
        mock_instantiate,
    },
    ContractError,
};
use astroport::router::{
    ExecuteMsg::ExecuteSwapOperations as AstroportExecuteSwapOperations, SwapOperation,
};
use cosmwasm_std::{
    attr, coin, coins,
    testing::{mock_info, MockQuerier, MOCK_CONTRACT_ADDR},
    to_binary, ContractResult, CosmosMsg, Querier, QuerierResult, QueryRequest, SystemResult,
    Uint128, WasmMsg,
};
use neutron_sdk::{
    bindings::{msg::IbcFee, query::NeutronQuery},
    query::min_ibc_fee::MinIbcFeeResponse,
};

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
fn private_method() {
    let (mut deps, env) = mock_instantiate::<MockQuerier>();
    let err = execute(
        deps.as_mut(),
        env,
        mock_info("stranger", &[]),
        craft_wrap_callback_msg(100, 300),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::InternalMethod {});
}

#[test]
fn zero_for_swap() {
    let (mut deps, env) = mock_instantiate::<MockQuerier>();
    let err = execute(
        deps.as_mut(),
        env,
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        craft_wrap_callback_msg(0, 300),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::ZeroForSwap {});
}

#[test]
fn not_enough_for_swap() {
    let (mut deps, env) = mock_instantiate::<MockQuerier>();
    let err = execute(
        deps.as_mut(),
        env,
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        craft_wrap_callback_msg(500, 300),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::NotEnoughForSwap {});
}

#[test]
fn success() {
    let (mut deps, env) = mock_instantiate::<CustomMockQuerier>();
    let response = execute(
        deps.as_mut(),
        env,
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        craft_wrap_callback_msg(100, 300),
    )
    .unwrap();

    assert_eq!(response.messages.len(), 2);
    assert_eq!(
        response.messages[0].msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "astroport_router".to_string(),
            msg: to_binary(&AstroportExecuteSwapOperations {
                operations: vec![SwapOperation::NativeSwap {
                    offer_denom: "canonical_denom".to_string(),
                    ask_denom: "ibc_fee_denom".to_string(),
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
        response.messages[1].msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_CONTRACT_ADDR.to_string(),
            msg: to_binary(&craft_swap_callback_msg()).unwrap(),
            funds: vec![],
        })
    );

    assert_eq!(
        response.attributes,
        vec![
            attr("amount_to_swap_for_ibc_fee", "100canonical_denom"),
            attr("amount_to_send", "200canonical_denom"),
            attr("min_ibc_fee", "50ibc_fee_denom"),
        ]
    );
}
