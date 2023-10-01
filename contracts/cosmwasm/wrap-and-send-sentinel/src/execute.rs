use crate::{
    contract::WRAP_AND_SEND_REPLY_ID,
    state::{CONFIG, FUNDS, REFUND_ADDRESS},
    ContractResult,
};
use cosmwasm_std::{to_binary, DepsMut, Env, MessageInfo, Response, SubMsg, WasmMsg};
use wrap_and_send::msg::ExecuteMsg;

pub fn execute_wrap_and_send(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    let refund_address = match &msg {
        ExecuteMsg::WrapAndSend { refund_address, .. } => deps.api.addr_validate(refund_address)?,
        _ => unreachable!(),
    };
    REFUND_ADDRESS.save(deps.storage, &refund_address)?;
    FUNDS.save(deps.storage, &info.funds)?;

    let msg = WasmMsg::Execute {
        contract_addr: config.wrap_and_send.into_string(),
        msg: to_binary(&msg)?,
        funds: info.funds,
    };

    // TODO: attributes
    Ok(Response::new().add_submessage(SubMsg::reply_always(msg, WRAP_AND_SEND_REPLY_ID)))
}
