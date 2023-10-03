use crate::{
    state::{CONFIG, FUNDS, REFUND_ADDRESS},
    ContractResult,
};
use cosmwasm_std::{to_binary, BankMsg, CosmosMsg, DepsMut, Env, Response, SubMsgResult, WasmMsg};
use lido_satellite::msg::ExecuteMsg::Mint as LidoSatelliteExecuteMint;

pub fn reply_wrap_and_send(
    deps: DepsMut,
    _env: Env,
    result: SubMsgResult,
) -> ContractResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    match result {
        // I ignore this error string since I am not sure how to propogate it
        // and inserting it into attributes doesn't sound right at all
        SubMsgResult::Err(_e) => {
            let refund_address = REFUND_ADDRESS.load(deps.storage)?;
            let mut funds = FUNDS.load(deps.storage)?;

            let wrap_msg: CosmosMsg = WasmMsg::Execute {
                contract_addr: config.lido_satellite.into_string(),
                msg: to_binary(&LidoSatelliteExecuteMint { receiver: None })?,
                funds: vec![funds.clone()],
            }
            .into();

            funds.denom = config.canonical_denom;
            let send_msg: CosmosMsg = BankMsg::Send {
                to_address: refund_address.into_string(),
                amount: vec![funds],
            }
            .into();

            // TODO: attributes
            Ok(Response::new().add_messages([wrap_msg, send_msg]))
        }
        // We don't need to do anything, our job is done
        // TODO: attributes
        SubMsgResult::Ok(_response) => Ok(Response::new()),
    }
}
