use crate::{
    contract::sudo,
    state::IBC_TRANSFER_INFO,
    tests::helpers::{craft_request_packet, instantiate_wrapper, prepare_ibc_transfer_info},
};
use cosmwasm_std::{attr, coin, testing::MockQuerier, BankMsg, CosmosMsg};
use neutron_sdk::sudo::msg::SudoMsg;

#[test]
fn test() {
    let (_result, mut deps, env) =
        instantiate_wrapper::<MockQuerier>("lido_satellite", "astroport_router");
    prepare_ibc_transfer_info(deps.as_mut());
    let response = sudo(
        deps.as_mut(),
        env,
        SudoMsg::Timeout {
            request: craft_request_packet(),
        },
    )
    .unwrap();
    assert_eq!(response.messages.len(), 1);
    assert_eq!(
        response.messages[0].msg,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "refund_address".to_string(),
            amount: vec![coin(500, "canonical_denom"), coin(20, "ibc_fee_denom")],
        })
    );
    assert_eq!(
        response.attributes,
        vec![
            attr("action", "ibc_timeout"),
            attr("action", "refund_sent_funds"),
            attr("amount", "500canonical_denom"),
            attr("action", "refund_ibc_ack_fee"),
            attr("amount", "20ibc_fee_denom"),
        ]
    );
    let ibc_transfer_info = IBC_TRANSFER_INFO
        .may_load(deps.as_mut().storage, (4, "chan"))
        .unwrap();
    assert_eq!(ibc_transfer_info, None);
}
