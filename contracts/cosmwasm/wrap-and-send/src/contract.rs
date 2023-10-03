use crate::{
    execute::{execute_swap_callback, execute_wrap_and_send},
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    query::query_config,
    reply::reply_ibc_transfer,
    state::{Config, CONFIG},
    sudo::{sudo_error, sudo_response, sudo_timeout},
    ContractError, ContractResult,
};
use cosmwasm_std::{attr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response};
use cw2::set_contract_version;
use lido_satellite::msg::{
    ConfigResponse as LidoSatelliteConfigResponse, QueryMsg::Config as LidoSatelliteQueryConfig,
};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    sudo::msg::SudoMsg,
};

pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) const IBC_TRANSFER_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let lido_satellite = deps.api.addr_validate(&msg.lido_satellite)?;
    let astroport_router = deps.api.addr_validate(&msg.astroport_router)?;
    let lido_satellite_config: LidoSatelliteConfigResponse = deps
        .querier
        .query_wasm_smart(&lido_satellite, &LidoSatelliteQueryConfig {})?;

    let config = Config {
        lido_satellite,
        astroport_router,
        bridged_denom: lido_satellite_config.bridged_denom,
        canonical_denom: lido_satellite_config.canonical_denom,
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attributes([
        attr("action", "instantiate"),
        attr("lido_satellite", config.lido_satellite),
        attr("astroport_router", config.astroport_router),
        attr("bridged_denom", config.bridged_denom),
        attr("canonical_denom", config.canonical_denom),
    ]))
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
            amount_to_swap_for_ibc_fee,
            ibc_fee_denom,
            astroport_swap_operations,
            refund_address,
        } => execute_wrap_and_send(
            deps,
            env,
            info,
            source_port,
            source_channel,
            receiver,
            amount_to_swap_for_ibc_fee,
            ibc_fee_denom,
            astroport_swap_operations,
            refund_address,
        ),
        ExecuteMsg::SwapCallback {
            source_port,
            source_channel,
            receiver,
            amount_to_send,
            min_ibc_fee,
            refund_address,
        } => execute_swap_callback(
            deps,
            env,
            info,
            source_port,
            source_channel,
            receiver,
            amount_to_send,
            min_ibc_fee,
            refund_address,
        ),
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
) -> ContractResult<Response<NeutronMsg>> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn reply(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    msg: Reply,
) -> ContractResult<Response<NeutronMsg>> {
    match msg.id {
        IBC_TRANSFER_REPLY_ID => reply_ibc_transfer(deps, env, msg.result),
        id => Err(ContractError::UnknownReplyId { id }),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn sudo(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    msg: SudoMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        SudoMsg::Response { request, data } => sudo_response(deps, env, request, data),
        SudoMsg::Error { request, details } => sudo_error(deps, env, request, details),
        SudoMsg::Timeout { request } => sudo_timeout(deps, env, request),
        _ => Ok(Response::new()),
    }
}
