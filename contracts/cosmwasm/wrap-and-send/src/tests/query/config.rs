use crate::{
    contract::query,
    msg::{ConfigResponse, QueryMsg},
    tests::helpers::mock_instantiate,
};
use cosmwasm_std::{from_binary, testing::MockQuerier};

#[test]
fn corresponds_to_instantiate_params() {
    let (deps, env) = mock_instantiate::<MockQuerier>();
    let config_response: ConfigResponse =
        from_binary(&query(deps.as_ref(), env, QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!(
        config_response,
        ConfigResponse {
            lido_satellite: "lido_satellite".to_string(),
            astroport_router: "astroport_router".to_string(),
            bridged_denom: "bridged_denom".to_string(),
            canonical_denom: "canonical_denom".to_string(),
        }
    )
}
