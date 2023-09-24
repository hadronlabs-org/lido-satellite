use crate::{
    contract::query,
    msg::{ConfigResponse, QueryMsg},
    tests::helpers::instantiate_wrapper,
};
use cosmwasm_std::{from_binary, testing::MockQuerier};

#[test]
fn corresponds_to_instantiate_params() {
    let (_result, deps, env) =
        instantiate_wrapper::<MockQuerier>("lido_satellite", "astroport_router");
    let config_response: ConfigResponse =
        from_binary(&query(deps.as_ref(), env, QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!(
        config_response,
        ConfigResponse {
            lido_satellite: "lido_satellite".to_string(),
            astroport_router: "astroport_router".to_string()
        }
    )
}
