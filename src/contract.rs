use crate::{
    execute::{execute_burn, execute_mint, execute_update_config},
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    query::query_config,
    state::{Config, CONFIG},
    ContractResult,
};
use cosmwasm_std::{attr, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;
use neutron_sdk::bindings::msg::NeutronMsg;

const CONTRACT_NAME: &str = "crates.io:lido-satellite";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
#[cfg_attr(feature = "interface", cw_orch::interface_entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    msg.validate()?;
    let owner = msg
        .owner
        .map_or(Ok(info.sender), |addr| deps.api.addr_validate(&addr))?;

    let config = Config {
        wsteth_denom: msg.wsteth_denom,
        subdenom: msg.subdenom,
        owner,
    };
    CONFIG.save(deps.storage, &config)?;

    let create_denom_msg = NeutronMsg::submit_create_denom(&config.subdenom);

    Ok(Response::new()
        .add_message(create_denom_msg)
        .add_attributes([
            attr("wsteth_denom", config.wsteth_denom),
            attr("subdenom", config.subdenom),
            attr("owner", config.owner.into_string()),
        ]))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
#[cfg_attr(feature = "interface", cw_orch::interface_entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::Mint { receiver } => execute_mint(deps, env, info, receiver),
        ExecuteMsg::Burn { receiver } => execute_burn(deps, env, info, receiver),
        ExecuteMsg::UpdateConfig {
            wsteth_denom,
            subdenom,
            owner,
        } => execute_update_config(deps, info, wsteth_denom, subdenom, owner),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
#[cfg_attr(feature = "interface", cw_orch::interface_entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Config {} => query_config(deps),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
#[cfg_attr(feature = "interface", cw_orch::interface_entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult<Response> {
    Ok(Response::new())
}
