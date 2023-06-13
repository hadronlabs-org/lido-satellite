use crate::{msg::ConfigResponse, state::CONFIG, ContractResult};
use cosmwasm_std::{to_binary, Binary, Deps};

pub(crate) fn query_config(deps: Deps) -> ContractResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    Ok(to_binary(&ConfigResponse {
        wsteth_denom: config.wsteth_denom,
        subdenom: config.subdenom,
    })?)
}
