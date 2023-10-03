use crate::{
    contract::sudo,
    state::IBC_TRANSFER_INFO,
    tests::helpers::{craft_request_packet, mock_instantiate, prepare_ibc_transfer_info},
};
use cosmwasm_std::{attr, coins, testing::MockQuerier, BankMsg, CosmosMsg};
use neutron_sdk::sudo::msg::SudoMsg;

#[test]
fn test() {
    let (mut deps, env) = mock_instantiate::<MockQuerier>();
    prepare_ibc_transfer_info(deps.as_mut());
    let response = sudo(
        deps.as_mut(),
        env,
        SudoMsg::Response {
            request: craft_request_packet(),
            data: Default::default(),
        },
    )
    .unwrap();
    assert_eq!(response.messages.len(), 1);
    assert_eq!(
        response.messages[0].msg,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "refund_address".to_string(),
            amount: coins(30, "ibc_fee_denom"),
        })
    );
    assert_eq!(
        response.attributes,
        vec![
            attr("action", "ibc_ack"),
            attr("status", "success"),
            attr("action", "refund_ibc_timeout_fee"),
            attr("amount", "30ibc_fee_denom"),
        ]
    );
    let ibc_transfer_info = IBC_TRANSFER_INFO
        .may_load(deps.as_mut().storage, (4, "chan"))
        .unwrap();
    assert_eq!(ibc_transfer_info, None);
}
