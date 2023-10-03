use crate::{
    contract::WRAP_AND_SEND_REPLY_ID,
    state::{CONFIG, FUNDS, REFUND_ADDRESS},
    ContractResult,
};
use cosmwasm_std::{to_binary, BankMsg, DepsMut, Env, MessageInfo, Response, SubMsg, WasmMsg};
use wrap_and_send::msg::ExecuteMsg;

pub fn execute_wrap_and_send(
    deps: DepsMut,
    _env: Env,
    mut info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    let refund_address = match &msg {
        ExecuteMsg::WrapAndSend { refund_address, .. } => deps.api.addr_validate(refund_address)?,
        _ => unreachable!(),
    };
    REFUND_ADDRESS.save(deps.storage, &refund_address)?;

    let funds = match info.funds.len() {
        1 => {
            let funds = info.funds.pop().unwrap();
            if funds.denom == config.bridged_denom {
                funds
            } else {
                // TODO: attributes
                return Ok(Response::new().add_message(BankMsg::Send {
                    to_address: refund_address.into_string(),
                    amount: vec![funds],
                }));
            }
        }
        _ => {
            // TODO: attributes
            return Ok(Response::new().add_message(BankMsg::Send {
                to_address: refund_address.into_string(),
                amount: info.funds,
            }));
        }
    };
    FUNDS.save(deps.storage, &funds)?;

    let msg = WasmMsg::Execute {
        contract_addr: config.wrap_and_send.into_string(),
        msg: to_binary(&msg)?,
        funds: vec![funds],
    };

    // TODO: attributes
    Ok(Response::new().add_submessage(SubMsg::reply_always(msg, WRAP_AND_SEND_REPLY_ID)))
}
