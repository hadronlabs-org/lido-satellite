use crate::{
    contract::ASTROPORT_SWAP_REPLY_ID,
    state::{CONFIG, IBC_FEE, WRAP_AND_SEND_CONTEXT},
    ContractResult,
};
use astroport::router::ExecuteMsg::ExecuteSwapOperations as AstroportExecuteSwapOperations;
use cosmwasm_std::{
    attr, coin, to_binary, BankMsg, DepsMut, Env, Response, SubMsg, SubMsgResult, WasmMsg,
};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    query::min_ibc_fee::query_min_ibc_fee,
};

pub(crate) fn reply_lido_satellite_wrap(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    result: SubMsgResult,
) -> ContractResult<Response<NeutronMsg>> {
    // Step 2.
    // Handle reply from Lido Satellite
    // On failure: refund bridged funds back to user
    // On success: swap part of canonical funds for IBC fee

    let config = CONFIG.load(deps.storage)?;
    let context = WRAP_AND_SEND_CONTEXT.load(deps.storage)?;

    match result {
        // I ignore this error string since I am not sure how to propogate it
        // and inserting it into attributes doesn't sound right at all
        SubMsgResult::Err(_e) => Ok(Response::new()
            .add_message(BankMsg::Send {
                to_address: context.refund_address.into_string(),
                amount: vec![context.amount_to_wrap],
            })
            .add_attributes([
                attr("action", "cancel_wrap_and_send"),
                attr("reason", "lido_satellite_wrap_failed"),
            ])),
        // Lido Satellite doesn't set data in Response, and we don't have to access events either
        // This means we can safely ignore reply response
        SubMsgResult::Ok(_response) => {
            let ibc_fee = {
                let mut fee = query_min_ibc_fee(deps.as_ref())?.min_fee;
                // fee.recv_fee is always empty
                fee.ack_fee
                    .retain(|coin| coin.denom == context.amount_to_swap_for_ibc_fee.denom);
                fee.timeout_fee
                    .retain(|coin| coin.denom == context.amount_to_swap_for_ibc_fee.denom);
                fee
            };
            IBC_FEE.save(deps.storage, &ibc_fee)?;

            let swap_msg = WasmMsg::Execute {
                contract_addr: config.astroport_router.into_string(),
                msg: to_binary(&AstroportExecuteSwapOperations {
                    operations: context.astroport_swap_operations,
                    minimum_receive: Some(
                        ibc_fee.ack_fee[0].amount + ibc_fee.timeout_fee[0].amount,
                    ),
                    to: None,
                    max_spread: None,
                })?,
                funds: vec![context.amount_to_swap_for_ibc_fee],
            };

            Ok(Response::new()
                .add_submessage(SubMsg::reply_always(swap_msg, ASTROPORT_SWAP_REPLY_ID))
                .add_attributes([
                    attr("action", "wrap_and_send"),
                    attr("subaction", "lido_satellite_wrap"),
                ]))
        }
    }
}

pub(crate) fn reply_astroport_swap(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    result: SubMsgResult,
) -> ContractResult<Response<NeutronMsg>> {
    // Step 3.
    // Handle reply from Astroport Router
    // On failure: refund canonical funds back to user
    // On success: refund excess IBC fee denom and perform IBC transfer to destination chain

    let config = CONFIG.load(deps.storage)?;
    let context = WRAP_AND_SEND_CONTEXT.load(deps.storage)?;
    let ibc_fee = IBC_FEE.load(deps.storage)?;

    match result {
        // I ignore this error string since I am not sure how to propogate it
        // and inserting it into attributes doesn't sound right at all
        // FIXME: what if, since we don't revert tx, funds get stuck on Astroport Router?
        SubMsgResult::Err(_e) => Ok(Response::new()
            .add_message(BankMsg::Send {
                to_address: context.refund_address.into_string(),
                amount: vec![coin(
                    context.amount_to_wrap.amount.u128(),
                    context.amount_to_send.denom,
                )],
            })
            .add_attributes([
                attr("action", "cancel_wrap_and_send"),
                attr("reason", "astroport_router_swap_failed"),
            ])),
        // Astroport Router doesn't set data in Response, and we don't have to access events either
        // This means we can safely ignore reply response
        SubMsgResult::Ok(_response) => {
            // TODO: 1. query own balance
            // TODO: 2. check if there are excess IBC fee denom
            // TODO: 3. launch IBC transfer
            // TODO: 4. refund excess IBC fee denom
            Ok(Response::new())
        }
    }
}
