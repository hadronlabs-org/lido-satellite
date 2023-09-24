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
    to_binary, BalanceResponse, BankMsg, BankQuery, Coin, ContractResult, CosmosMsg, DepsMut,
    Empty, Env, Querier, QuerierResult, QueryRequest, Reply, Response, SubMsgResponse,
    SubMsgResult, SystemResult, Uint128,
};
use neutron_sdk::{
    bindings::{
        msg::{IbcFee, NeutronMsg},
        query::NeutronQuery,
    },
    sudo::msg::RequestPacketTimeoutHeight,
};

fn handle_query(bin_request: &[u8], balance: u128) -> QuerierResult {
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
                        amount: Uint128::new(balance),
                    },
                })))
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}

#[derive(Default)]
struct MockQuerierLessFunds {}

impl Querier for MockQuerierLessFunds {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        handle_query(bin_request, 40)
    }
}

#[derive(Default)]
struct MockQuerierExactFunds {}

impl Querier for MockQuerierExactFunds {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        handle_query(bin_request, 50)
    }
}

#[derive(Default)]
struct MockQuerierMoreFunds {}

impl Querier for MockQuerierMoreFunds {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        handle_query(bin_request, 62)
    }
}

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
    let response = reply_helper(deps.as_mut(), env);
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

#[test]
fn received_exact_amount_of_funds_from_astroport_as_expected() {
    let (_result, mut deps, env) =
        instantiate_wrapper::<MockQuerierExactFunds>("lido_satellite", "astroport_router");
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
    let response = reply_helper(deps.as_mut(), env.clone());
    assert_eq!(response.messages.len(), 1);
    assert_eq!(response.messages[0].msg, ibc_message_helper(&env));
    assert_eq!(
        response.attributes,
        vec![
            attr("subaction", "astroport_router_swap"),
            attr("swapped_amount", "50"),
            attr("subaction", "perform_ibc_transfer"),
            attr("source_port", "source_port"),
            attr("source_channel", "source_channel"),
            attr("token", "200canonical_denom"),
            attr("sender", MOCK_CONTRACT_ADDR),
            attr("receiver", "receiver"),
            attr("timeout_height", "null"),
            attr(
                "timeout_timestamp",
                env.block.time.plus_minutes(20).nanos().to_string()
            ),
        ]
    );
}

#[test]
fn received_more_funds_from_astroport_than_expected() {
    let (_result, mut deps, env) =
        instantiate_wrapper::<MockQuerierMoreFunds>("lido_satellite", "astroport_router");
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
    let response = reply_helper(deps.as_mut(), env.clone());
    assert_eq!(response.messages.len(), 2);
    assert_eq!(response.messages[0].msg, ibc_message_helper(&env),);
    assert_eq!(
        response.messages[1].msg,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "refund_address".to_string(),
            amount: coins(12, "ibc_fee_denom")
        })
    );
    assert_eq!(
        response.attributes,
        vec![
            attr("subaction", "astroport_router_swap"),
            attr("swapped_amount", "62"),
            attr("subaction", "perform_ibc_transfer"),
            attr("source_port", "source_port"),
            attr("source_channel", "source_channel"),
            attr("token", "200canonical_denom"),
            attr("sender", MOCK_CONTRACT_ADDR),
            attr("receiver", "receiver"),
            attr("timeout_height", "null"),
            attr(
                "timeout_timestamp",
                env.block.time.plus_minutes(20).nanos().to_string()
            ),
            attr("subaction", "refund_excess_swapped_fee"),
            attr("amount", "12ibc_fee_denom"),
        ]
    );
}

fn reply_helper(deps: DepsMut<NeutronQuery>, env: Env) -> Response<NeutronMsg> {
    reply(
        deps,
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
    .unwrap()
}

fn ibc_message_helper(env: &Env) -> CosmosMsg<NeutronMsg> {
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
        timeout_timestamp: env.block.time.plus_minutes(20).nanos(),
        memo: "".to_string(),
        fee: IbcFee {
            recv_fee: vec![],
            ack_fee: coins(20, "ibc_fee_denom"),
            timeout_fee: coins(30, "ibc_fee_denom"),
        },
    })
}
