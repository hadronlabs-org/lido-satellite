use crate::{contract::execute, msg::ExecuteMsg, tests::helpers::instantiate_wrapper};
use cosmwasm_std::{testing::mock_info, Uint128};

#[test]
fn no_funds() {
    // TODO: mock query lido satellite config
    let (_result, mut deps, env) = instantiate_wrapper("lido_satellite", "aatroport_router");
    let err = execute(
        deps.as_mut(),
        env,
        mock_info("stranger", &[]),
        ExecuteMsg::WrapAndSend {
            source_port: "source_port".to_string(),
            source_channel: "source_channel".to_string(),
            receiver: "receiver".to_string(),
            amount_to_swap_for_ibc_fee: Uint128::zero(),
            ibc_fee_denom: "ibc_fee_denom".to_string(),
            astroport_swap_operations: vec![],
            refund_address: "refund_address".to_string(),
        },
    )
    .unwrap_err();
    dbg!(err);
}
