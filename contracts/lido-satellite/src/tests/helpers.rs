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

pub const VALID_IBC_DENOM: &str =
    "ibc/584A4A23736884E0C198FD1EE932455A9357A492A7B94324E4A02B5628687831";

pub fn instantiate_wrapper(
    bridged_denom: impl Into<String>,
    canonical_subdenom: impl Into<String>,
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
                bridged_denom: bridged_denom.into(),
                canonical_subdenom: canonical_subdenom.into(),
            },
        ),
        deps,
        env,
    )
}

pub fn assert_config(deps: Deps, bridged_denom: &str, canonical_subdenom: &str) {
    let config = CONFIG.load(deps.storage).unwrap();
    assert_eq!(
        config,
        Config {
            bridged_denom: bridged_denom.to_string(),
            canonical_subdenom: canonical_subdenom.to_string(),
        }
    )
}
