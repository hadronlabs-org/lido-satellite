use crate::{
    contract::{reply, IBC_TRANSFER_REPLY_ID},
    state::{IBC_FEE, WRAP_AND_SEND_CONTEXT},
    tests::helpers::{craft_wrap_and_send_context, instantiate_wrapper},
};
use cosmwasm_std::{
    attr, coin, coins, testing::MockQuerier, to_binary, BankMsg, CosmosMsg, Reply, SubMsgResponse,
    SubMsgResult,
};
use neutron_sdk::bindings::msg::{IbcFee, MsgIbcTransferResponse};

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
            id: IBC_TRANSFER_REPLY_ID,
            // we don't use this error string anyway, so we can put anything in there
            result: SubMsgResult::Err("ibc_transfer_error".to_string()),
        },
    )
    .unwrap();
    assert_eq!(response.messages.len(), 1);
    assert_eq!(
        response.messages[0].msg,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "refund_address".to_string(),
            amount: vec![coin(200, "canonical_denom"), coin(50, "ibc_fee_denom")]
        })
    );
    assert_eq!(
        response.attributes,
        vec![
            attr("action", "cancel_wrap_and_send"),
            attr("reason", "ibc_transfer_failed"),
        ]
    );
}

#[test]
fn success() {
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
            id: IBC_TRANSFER_REPLY_ID,
            result: SubMsgResult::Ok(SubMsgResponse {
                // we completely ignore events, so we can leave this field empty
                events: vec![],
                data: Some(
                    to_binary(&MsgIbcTransferResponse {
                        sequence_id: 7,
                        channel: "some_channel".to_string(),
                    })
                    .unwrap(),
                ),
            }),
        },
    )
    .unwrap();
    assert_eq!(response.messages.len(), 0);
    assert_eq!(
        response.attributes,
        vec![
            attr("subaction", "ibc_transfer"),
            attr("sequence_id", "7"),
            attr("channel", "some_channel"),
        ]
    );
}
