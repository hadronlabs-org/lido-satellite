use crate::{
    execute::{execute_burn, execute_mint},
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    query::query_config,
    state::{Config, CONFIG},
    ContractError, ContractResult,
};
use cosmwasm_std::{attr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, SubMsg};
use cw2::set_contract_version;
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    query::token_factory::query_full_denom,
};

pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const CREATE_DENOM_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    msg.validate()?;
    let config = Config {
        bridged_denom: msg.bridged_denom,
        // we save here just a plain subdenom,
        // which will be updated to a full denom later in the reply handler
        canonical_denom: msg.canonical_subdenom,
    };
    CONFIG.save(deps.storage, &config)?;

    let create_denom_msg = NeutronMsg::submit_create_denom(&config.canonical_denom);
    let create_denom_submsg = SubMsg::reply_on_success(create_denom_msg, CREATE_DENOM_REPLY_ID);
    Ok(Response::new()
        .add_submessage(create_denom_submsg)
        .add_attributes([
            attr("bridged_denom", config.bridged_denom),
            attr("canonical_subdenom", config.canonical_denom),
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
        ExecuteMsg::Mint { receiver } => execute_mint(deps, env, info, receiver),
        ExecuteMsg::Burn { receiver } => execute_burn(deps, env, info, receiver),
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
pub fn reply(deps: DepsMut<NeutronQuery>, env: Env, msg: Reply) -> ContractResult<Response> {
    match msg.id {
        CREATE_DENOM_REPLY_ID => {
            // at this point, `config.canonical_denom` stores just a subdenom
            let mut config = CONFIG.load(deps.storage)?;

            let full_denom =
                query_full_denom(deps.as_ref(), env.contract.address, config.canonical_denom)?;

            // but we replace it with a full denom at this step, like it should be
            config.canonical_denom = full_denom.denom;

            CONFIG.save(deps.storage, &config)?;
            Ok(Response::new().add_attribute("canonical_denom", config.canonical_denom))
        }
        id => Err(ContractError::UnknownReplyId { id }),
    }
}
