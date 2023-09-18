use crate::{contract::instantiate, msg::InstantiateMsg, ContractResult};
use cosmwasm_std::{
    testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage},
    Env, OwnedDeps, Response,
};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};
use std::marker::PhantomData;

#[allow(clippy::type_complexity)]
pub fn instantiate_wrapper(
    lido_satellite: impl Into<String>,
    ibc_fee_denom: impl Into<String>,
    owner: Option<String>,
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
                lido_satellite: lido_satellite.into(),
                ibc_fee_denom: ibc_fee_denom.into(),
                owner,
            },
        ),
        deps,
        env,
    )
}
