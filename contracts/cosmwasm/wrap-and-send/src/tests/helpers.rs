use crate::{
    contract::instantiate,
    msg::InstantiateMsg,
    state::{Config, CONFIG},
    ContractResult,
};
use cosmwasm_std::{
    testing::{mock_env, mock_info, MockApi, MockStorage},
    Addr, Deps, Env, OwnedDeps, Querier, Response,
};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};
use std::marker::PhantomData;

#[allow(clippy::type_complexity)]
pub fn instantiate_wrapper<Q: Querier + Default>(
    lido_satellite: impl Into<String>,
    astroport_router: impl Into<String>,
) -> (
    ContractResult<Response<NeutronMsg>>,
    OwnedDeps<MockStorage, MockApi, Q, NeutronQuery>,
    Env,
) {
    let mut deps = OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: Q::default(),
        custom_query_type: PhantomData,
    };
    let env = mock_env();
    (
        instantiate(
            deps.as_mut(),
            env.clone(),
            mock_info("admin", &[]),
            InstantiateMsg {
                lido_satellite: lido_satellite.into(),
                astroport_router: astroport_router.into(),
            },
        ),
        deps,
        env,
    )
}

pub fn assert_config(deps: Deps<NeutronQuery>, lido_satellite: &str, astroport_router: &str) {
    let config = CONFIG.load(deps.storage).unwrap();
    assert_eq!(
        config,
        Config {
            lido_satellite: Addr::unchecked(lido_satellite),
            astroport_router: Addr::unchecked(astroport_router),
        }
    )
}
