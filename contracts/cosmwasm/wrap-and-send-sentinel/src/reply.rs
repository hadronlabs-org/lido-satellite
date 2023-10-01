use crate::state::{FUNDS, REFUND_ADDRESS};
use crate::ContractResult;
use cosmwasm_std::{BankMsg, DepsMut, Env, Response, SubMsgResult};

pub fn reply_wrap_and_send(
    deps: DepsMut,
    _env: Env,
    result: SubMsgResult,
) -> ContractResult<Response> {
    match result {
        // I ignore this error string since I am not sure how to propogate it
        // and inserting it into attributes doesn't sound right at all
        SubMsgResult::Err(_e) => {
            let refund_address = REFUND_ADDRESS.load(deps.storage)?;
            let funds = FUNDS.load(deps.storage)?;
            // TODO: attributes
            // TODO: wrap and refund
            Ok(Response::new().add_message(BankMsg::Send {
                to_address: refund_address.into_string(),
                amount: funds,
            }))
        }
        // TODO: attributes
        // We don't need to do anything, our job is done
        SubMsgResult::Ok(_response) => Ok(Response::new()),
    }
}
