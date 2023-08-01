use crate::{
    contract::instantiate,
    msg::InstantiateMsg,
    state::{Config, CONFIG},
    ContractResult,
};
use cosmwasm_std::{
    testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage},
    Deps, Env, OwnedDeps, Response,
};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};
use std::marker::PhantomData;

pub const VALID_IBC_DENOM: &str =
    "ibc/584A4A23736884E0C198FD1EE932455A9357A492A7B94324E4A02B5628687831";

#[allow(clippy::type_complexity)]
pub fn instantiate_wrapper(
    bridged_denom: impl Into<String>,
    canonical_subdenom: impl Into<String>,
) -> (
    ContractResult<Response<NeutronMsg>>,
    OwnedDeps<MockStorage, MockApi, MockQuerier, NeutronQuery>,
    Env,
) {
    let mut deps = OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: MockQuerier::default(),
        custom_query_type: PhantomData,
    };
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

pub fn assert_config(deps: Deps<NeutronQuery>, bridged_denom: &str, canonical_denom: &str) {
    let config = CONFIG.load(deps.storage).unwrap();
    assert_eq!(
        config,
        Config {
            bridged_denom: bridged_denom.to_string(),
            canonical_denom: canonical_denom.to_string(),
        }
    )
}
