use crate::{
    contract::instantiate,
    msg::InstantiateMsg,
    state::{Config, CONFIG},
    ContractResult,
};
use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage},
    Deps, Env, OwnedDeps, Response,
};
use neutron_sdk::bindings::msg::NeutronMsg;

pub fn instantiate_wrapper(
    wsteth_denom: impl Into<String>,
    subdenom: impl Into<String>,
) -> (
    ContractResult<Response<NeutronMsg>>,
    OwnedDeps<MockStorage, MockApi, MockQuerier>,
    Env,
) {
    let mut deps = mock_dependencies();
    let env = mock_env();
    (
        instantiate(
            deps.as_mut(),
            env.clone(),
            mock_info("admin", &[]),
            InstantiateMsg {
                wsteth_denom: wsteth_denom.into(),
                subdenom: subdenom.into(),
            },
        ),
        deps,
        env,
    )
}

pub fn assert_config(deps: Deps, wsteth_denom: &str, subdenom: &str) {
    let config = CONFIG.load(deps.storage).unwrap();
    assert_eq!(
        config,
        Config {
            wsteth_denom: wsteth_denom.to_string(),
            subdenom: subdenom.to_string(),
        }
    )
}
