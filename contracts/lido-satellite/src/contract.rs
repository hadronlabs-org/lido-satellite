use crate::{
    execute::{execute_burn, execute_mint},
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    query::query_config,
    state::{Config, CONFIG},
    ContractResult,
};
use cosmwasm_std::{attr, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;
use neutron_sdk::bindings::msg::NeutronMsg;

pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    msg.validate()?;
    let config = Config {
        bridged_denom: msg.bridged_denom,
        canonical_subdenom: msg.canonical_subdenom,
    };
    CONFIG.save(deps.storage, &config)?;

    let create_denom_msg = NeutronMsg::submit_create_denom(&config.canonical_subdenom);
    Ok(Response::new()
        .add_message(create_denom_msg)
        .add_attributes([
            attr("bridged_denom", config.bridged_denom),
            attr("canonical_subdenom", config.canonical_subdenom),
        ]))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::Mint { receiver } => execute_mint(deps, env, info, receiver),
        ExecuteMsg::Burn { receiver } => execute_burn(deps, env, info, receiver),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Config {} => query_config(deps),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult<Response> {
    Ok(Response::new())
}
