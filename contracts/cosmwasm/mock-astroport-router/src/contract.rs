use crate::{
    execute::execute_swap_operations,
    msg::{ExecuteMsg, InstantiateMsg},
    state::{Config, CONFIG},
    ContractResult,
};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        offer_denom: msg.offer_denom,
        ask_denom: msg.ask_denom,
    };
    CONFIG.save(deps.storage, &config)?;

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
        ExecuteMsg::ExecuteSwapOperations {
            operations,
            minimum_receive,
            to,
            max_spread,
        } => {
            let res = execute_swap_operations(
                &deps,
                env,
                info,
                operations,
                minimum_receive,
                to,
                max_spread,
            );
            deps.api.debug(&format!("WASMDEBUG: res: {:?}", &res));
            res
        }
        _ => unimplemented!(),
    }
}
