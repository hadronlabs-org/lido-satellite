use crate::{
    msg::InstantiateMsg,
    state::{Config, CONFIG},
    ContractResult,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;
use neutron_sdk::bindings::msg::NeutronMsg;

const CONTRACT_NAME: &str = "crates.io:lido-satellite";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
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
