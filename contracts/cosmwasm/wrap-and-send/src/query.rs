use crate::{msg::ConfigResponse, state::CONFIG, ContractResult};
use cosmwasm_std::{to_binary, Binary, Deps};
use neutron_sdk::bindings::query::NeutronQuery;

pub(crate) fn query_config(deps: Deps<NeutronQuery>) -> ContractResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    Ok(to_binary(&ConfigResponse {
        lido_satellite: config.lido_satellite.into_string(),
        astroport_router: config.astroport_router.into_string(),
    })?)
}
