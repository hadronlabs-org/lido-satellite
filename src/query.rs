use crate::{
    msg::{ConfigResponse, QueryMsg},
    state::CONFIG,
    ContractResult,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, Env};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Config {} => query_config(deps),
    }
}

fn query_config(deps: Deps) -> ContractResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    Ok(to_binary(&ConfigResponse {
        wsteth_denom: config.wsteth_denom,
        subdenom: config.subdenom,
        owner: config.owner.into_string(),
    })?)
}
