use crate::{
    contract::{reply, ASTROPORT_SWAP_REPLY_ID},
    state::{IBC_FEE, WRAP_AND_SEND_CONTEXT},
    tests::helpers::{
        bin_request_to_query_request, craft_wrap_and_send_context, instantiate_wrapper,
    },
};
use cosmwasm_std::{
    attr, coin, coins,
    testing::{MockQuerier, MOCK_CONTRACT_ADDR},
    to_binary, BalanceResponse, BankMsg, BankQuery, Coin, ContractResult, CosmosMsg, Empty,
    Querier, QuerierResult, QueryRequest, Reply, SubMsgResponse, SubMsgResult, SystemResult,
    Uint128,
};
use neutron_sdk::bindings::msg::IbcFee;

#[derive(Default)]
struct MockQuerierLessFunds {}

impl Querier for MockQuerierLessFunds {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request = match bin_request_to_query_request::<Empty>(bin_request) {
            Ok(v) => v,
            Err(e) => return e,
        };
        match request {
            QueryRequest::Bank(query) => match query {
                BankQuery::Balance { address, denom } => {
                    assert_eq!(address, MOCK_CONTRACT_ADDR);
                    assert_eq!(denom, "ibc_fee_denom");
                    SystemResult::Ok(ContractResult::from(to_binary(&BalanceResponse {
                        amount: Coin {
                            denom: "ibc_fee_denom".to_string(),
                            amount: Uint128::new(40),
                        },
                    })))
                }
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }
}

// TODO: 1. [x] astroport swap failed
// TODO: 2. [x] received less funds from astroport than expected
// TODO: 3. [ ] received exact amount of funds from astroport (check IBC message and attrs)
// TODO: 4. [ ] received more funds from astroport than expected (check IBC message, refund and attrs)

#[test]
fn fail() {
    let (_result, mut deps, env) =
        instantiate_wrapper::<MockQuerier>("lido_satellite", "astroport_router");
    WRAP_AND_SEND_CONTEXT
        .save(deps.as_mut().storage, &craft_wrap_and_send_context())
        .unwrap();
    IBC_FEE
        .save(
            deps.as_mut().storage,
            &IbcFee {
                recv_fee: vec![],
                ack_fee: coins(20, "ibc_fee_denom"),
                timeout_fee: coins(30, "ibc_fee_denom"),
            },
        )
        .unwrap();
    let response = reply(
        deps.as_mut(),
        env,
        Reply {
            id: ASTROPORT_SWAP_REPLY_ID,
            // we don't use this error string anyway, so we can put anything in there
            result: SubMsgResult::Err("mock_astroport_swap_error".to_string()),
        },
    )
    .unwrap();
    assert_eq!(response.messages.len(), 1);
    assert_eq!(
        response.messages[0].msg,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "refund_address".to_string(),
            amount: coins(300, "canonical_denom"),
        })
    );
    assert_eq!(
        response.attributes,
        vec![
            attr("action", "cancel_wrap_and_send"),
            attr("reason", "astroport_router_swap_failed")
        ]
    );
}

#[test]
fn received_less_funds_from_astroport_than_expected() {
    let (_result, mut deps, env) =
        instantiate_wrapper::<MockQuerierLessFunds>("lido_satellite", "astroport_router");
    WRAP_AND_SEND_CONTEXT
        .save(deps.as_mut().storage, &craft_wrap_and_send_context())
        .unwrap();
    IBC_FEE
        .save(
            deps.as_mut().storage,
            &IbcFee {
                recv_fee: vec![],
                ack_fee: coins(20, "ibc_fee_denom"),
                timeout_fee: coins(30, "ibc_fee_denom"),
            },
        )
        .unwrap();
    let response = reply(
        deps.as_mut(),
        env,
        Reply {
            id: ASTROPORT_SWAP_REPLY_ID,
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
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "refund_address".to_string(),
            amount: vec![coin(200, "canonical_denom"), coin(40, "ibc_fee_denom")],
        })
    );
    assert_eq!(
        response.attributes,
        vec![
            attr("action", "cancel_wrap_and_send"),
            attr("reason", "not_enough_fee_after_swap"),
        ]
    );
}
