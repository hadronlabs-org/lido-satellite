use crate::{
    contract::{reply, LIDO_SATELLITE_WRAP_REPLY_ID},
    state::{IBC_FEE, WRAP_AND_SEND_CONTEXT},
    tests::helpers::{
        bin_request_to_query_request, craft_wrap_and_send_context, instantiate_wrapper,
    },
};
use astroport::router::ExecuteMsg::ExecuteSwapOperations as AstroportExecuteSwapOperations;
use cosmwasm_std::{
    attr, coin, coins, testing::MockQuerier, to_binary, BankMsg, ContractResult, CosmosMsg,
    Querier, QuerierResult, QueryRequest, Reply, SubMsgResponse, SubMsgResult, SystemResult,
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
fn lido_satellite_wrap_failed() {
    let (_result, mut deps, env) =
        instantiate_wrapper::<MockQuerier>("lido_satellite", "astroport_router");
    WRAP_AND_SEND_CONTEXT
        .save(deps.as_mut().storage, &craft_wrap_and_send_context())
        .unwrap();
    let response = reply(
        deps.as_mut(),
        env,
        Reply {
            id: LIDO_SATELLITE_WRAP_REPLY_ID,
            // we don't use this error string anyway, so we can put anything in there
            result: SubMsgResult::Err("mock_lido_satellite_error".to_string()),
        },
    )
    .unwrap();
    assert_eq!(response.messages.len(), 1);
    assert_eq!(
        response.messages[0].msg,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "refund_address".to_string(),
            amount: coins(300, "bridged_denom"),
        })
    );
    assert_eq!(
        response.attributes,
        vec![
            attr("action", "cancel_wrap_and_send"),
            attr("reason", "lido_satellite_wrap_failed"),
        ]
    )
}

#[test]
fn success() {
    let (_result, mut deps, env) =
        instantiate_wrapper::<CustomMockQuerier>("lido_satellite", "astroport_router");
    WRAP_AND_SEND_CONTEXT
        .save(deps.as_mut().storage, &craft_wrap_and_send_context())
        .unwrap();
    let response = reply(
        deps.as_mut(),
        env,
        Reply {
            id: LIDO_SATELLITE_WRAP_REPLY_ID,
            result: SubMsgResult::Ok(SubMsgResponse {
                // we completely ignore events, so we can leave this field empty
                events: vec![],
                // Lido Satellite doesn't set this field, so we can leave it empty
                data: None,
            }),
        },
    )
    .unwrap();
    assert_eq!(response.messages.len(), 1);
    assert_eq!(
        response.messages[0].msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "astroport_router".to_string(),
            msg: to_binary(&AstroportExecuteSwapOperations {
                operations: vec![],
                minimum_receive: Some(Uint128::new(50)),
                to: None,
                max_spread: None,
            })
            .unwrap(),
            funds: coins(100, "canonical_denom"),
        })
    );
    assert_eq!(
        response.attributes,
        vec![attr("subaction", "lido_satellite_wrap")]
    );
    let ibc_fee = IBC_FEE.load(deps.as_mut().storage).unwrap();
    assert_eq!(
        ibc_fee,
        IbcFee {
            recv_fee: vec![],
            ack_fee: coins(20, "ibc_fee_denom"),
            timeout_fee: coins(30, "ibc_fee_denom"),
        }
    );
}
