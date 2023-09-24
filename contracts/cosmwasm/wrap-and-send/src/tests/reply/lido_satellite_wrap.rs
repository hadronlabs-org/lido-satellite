use crate::{
    contract::{reply, LIDO_SATELLITE_WRAP_REPLY_ID},
    state::WRAP_AND_SEND_CONTEXT,
    tests::helpers::{craft_wrap_and_send_context, instantiate_wrapper},
};
use cosmwasm_std::{attr, coins, testing::MockQuerier, BankMsg, CosmosMsg, Reply, SubMsgResult};

// TODO: required mocks:
//       1. minimum IBC fee
//       possible scenarios:
//       1. [x] Lido Satellite wrap failed (check refund and attrs)
//       2. [ ] everything is ok (check msg to astroport, attrs and if IBC_FEE is set correctly)

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
