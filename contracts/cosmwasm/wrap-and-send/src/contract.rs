use crate::{
    execute::execute_wrap_and_send,
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    query::query_config,
    state::{Config, CONFIG},
    ContractResult,
};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response};
use cw2::set_contract_version;
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};

pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let lido_satellite = deps.api.addr_validate(&msg.lido_satellite)?;
    let config = Config {
        lido_satellite,
        ibc_fee_denom: msg.ibc_fee_denom,
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("lido_satellite", msg.lido_satellite))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::WrapAndSend {
            source_port,
            source_channel,
            receiver,
        } => execute_wrap_and_send(deps, env, info, source_port, source_channel, receiver),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Config {} => query_config(deps),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(
    _deps: DepsMut<NeutronQuery>,
    _env: Env,
    _msg: MigrateMsg,
) -> ContractResult<Response> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn reply(_deps: DepsMut<NeutronQuery>, _env: Env, _msg: Reply) -> ContractResult<Response> {
    Ok(Response::new())
}
