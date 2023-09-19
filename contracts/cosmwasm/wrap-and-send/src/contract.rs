use crate::{
    execute::{
        execute_set_ibc_fee_denom, execute_set_owner, execute_withdraw_funds, execute_wrap_and_send,
    },
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    query::query_config,
    state::{Config, CONFIG},
    ContractResult,
};
use cosmwasm_std::{attr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response};
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
    let owner = msg
        .owner
        .map(|addr| deps.api.addr_validate(&addr))
        .transpose()?;
    let config = Config {
        lido_satellite,
        ibc_fee_denom: msg.ibc_fee_denom,
        owner,
    };
    CONFIG.save(deps.storage, &config)?;

    let mut attributes = vec![
        attr("lido_satellite", config.lido_satellite),
        attr("ibc_fee_denom", config.ibc_fee_denom),
    ];
    if let Some(owner) = config.owner {
        attributes.push(attr("owner", owner))
    }
    Ok(Response::new().add_attributes(attributes))
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
        ExecuteMsg::SetOwner { new_owner } => execute_set_owner(deps, env, info, new_owner),
        ExecuteMsg::SetIbcFeeDenom { new_ibc_fee_denom } => {
            execute_set_ibc_fee_denom(deps, env, info, new_ibc_fee_denom)
        }
        ExecuteMsg::WithdrawFunds { funds, receiver } => {
            execute_withdraw_funds(deps, env, info, funds, receiver)
        }
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
