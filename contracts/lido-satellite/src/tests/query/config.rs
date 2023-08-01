use crate::{
    contract::query,
    msg::{ConfigResponse, QueryMsg},
    tests::helpers::{instantiate_wrapper, VALID_IBC_DENOM},
};
use cosmwasm_std::from_binary;

#[test]
fn corresponds_to_instantiate_params() {
    let (_result, deps, env) = instantiate_wrapper(VALID_IBC_DENOM, "eth");
    let config_response: ConfigResponse =
        from_binary(&query(deps.as_ref(), env, QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!(
        config_response,
        ConfigResponse {
            bridged_denom: VALID_IBC_DENOM.to_string(),
            canonical_denom: "eth".to_string(),
        }
    );
}
