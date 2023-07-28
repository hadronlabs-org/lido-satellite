use crate::{
    contract::query,
    msg::{ConfigResponse, QueryMsg},
    tests::helpers::instantiate_wrapper,
};
use cosmwasm_std::from_binary;

#[test]
fn corresponds_to_instantiate_params() {
    let (_result, deps, env) = instantiate_wrapper("wsteth", "eth");
    let config_response: ConfigResponse =
        from_binary(&query(deps.as_ref(), env, QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!(
        config_response,
        ConfigResponse {
            bridged_denom: "wsteth".to_string(),
            canonical_subdenom: "eth".to_string(),
        }
    );
}
