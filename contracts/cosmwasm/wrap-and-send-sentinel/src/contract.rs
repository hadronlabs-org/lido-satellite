use crate::{
    execute::execute_wrap_and_send,
    msg::{ExecuteMsg, InstantiateMsg},
    reply::reply_wrap_and_send,
    state::{Config, CONFIG},
    ContractError, ContractResult,
};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Reply, Response};
use cw2::set_contract_version;
use wrap_and_send::msg::{
    ConfigResponse as WrapAndSendConfigResponse, QueryMsg::Config as WrapAndSendQueryConfig,
};

pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const WRAP_AND_SEND_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let wrap_and_send = deps.api.addr_validate(&msg.wrap_and_send)?;
    let wrap_and_send_config: WrapAndSendConfigResponse = deps
        .querier
        .query_wasm_smart(&wrap_and_send, &WrapAndSendQueryConfig {})?;
    let lido_satellite = deps
        .api
        .addr_validate(&wrap_and_send_config.lido_satellite)?;

    let config = Config {
        wrap_and_send,
        lido_satellite,
        bridged_denom: wrap_and_send_config.bridged_denom,
        canonical_denom: wrap_and_send_config.canonical_denom,
    };
    CONFIG.save(deps.storage, &config)?;

    // TODO: attributes
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::WrapAndSend { .. } => execute_wrap_and_send(deps, env, info, msg),
        _ => unimplemented!(),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> ContractResult<Response> {
    match msg.id {
        WRAP_AND_SEND_REPLY_ID => reply_wrap_and_send(deps, env, msg.result),
        id => Err(ContractError::UnknownReplyId { id }),
    }
}
