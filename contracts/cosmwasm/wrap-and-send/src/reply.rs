use crate::{
    contract::{ASTROPORT_SWAP_REPLY_ID, IBC_TRANSFER_REPLY_ID},
    state::{IbcTransferInfo, CONFIG, IBC_FEE, IBC_TRANSFER_INFO, WRAP_AND_SEND_CONTEXT},
    ContractResult,
};
use astroport::router::ExecuteMsg::ExecuteSwapOperations as AstroportExecuteSwapOperations;
use cosmwasm_std::{
    attr, coin, from_binary, to_binary, BankMsg, DepsMut, Env, Response, SubMsg, SubMsgResult,
    WasmMsg,
};
use neutron_sdk::{
    bindings::{
        msg::{MsgIbcTransferResponse, NeutronMsg},
        query::NeutronQuery,
    },
    query::min_ibc_fee::query_min_ibc_fee,
    sudo::msg::RequestPacketTimeoutHeight,
};
use std::cmp::Ordering;

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
                // FIXME: can this query ever fail?
                // FIXME: can this query ever return empty response or even invalid response?
                let mut fee = query_min_ibc_fee(deps.as_ref())?.min_fee;
                // fee.recv_fee is always empty
                fee.ack_fee
                    .retain(|coin| coin.denom == context.ibc_fee_denom);
                fee.timeout_fee
                    .retain(|coin| coin.denom == context.ibc_fee_denom);
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
                // FIXME: possible reentrance attack: if Astroport Router calls us, it will
                //        override our temporary state which we use between reply() calls
                .add_submessage(SubMsg::reply_always(swap_msg, ASTROPORT_SWAP_REPLY_ID))
                .add_attributes([attr("subaction", "lido_satellite_wrap")]))
        }
    }
}

pub(crate) fn reply_astroport_swap(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    result: SubMsgResult,
) -> ContractResult<Response<NeutronMsg>> {
    // Step 3.
    // Handle reply from Astroport Router
    // On failure: refund canonical funds back to user
    // On success: refund excess IBC fee denom and perform IBC transfer to destination chain

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
            let fee_balance = deps
                .querier
                .query_balance(&env.contract.address, &context.ibc_fee_denom)?;
            let refund = match fee_balance
                .amount
                .cmp(&(ibc_fee.ack_fee[0].amount + ibc_fee.timeout_fee[0].amount))
            {
                Ordering::Less => {
                    // should never happen, but let's be cautious
                    return Ok(Response::new()
                        .add_message(BankMsg::Send {
                            to_address: context.refund_address.into_string(),
                            amount: vec![context.amount_to_send, fee_balance],
                        })
                        .add_attributes([
                            attr("action", "cancel_wrap_and_send"),
                            attr("reason", "not_enough_fee_after_swap"),
                        ]));
                }
                Ordering::Equal => None,
                Ordering::Greater => Some(
                    fee_balance.amount - ibc_fee.ack_fee[0].amount - ibc_fee.timeout_fee[0].amount,
                ),
            };

            let timeout_timestamp = env.block.time.plus_minutes(20).nanos();
            let ibc_transfer = NeutronMsg::IbcTransfer {
                source_port: context.source_port.clone(),
                source_channel: context.source_channel.clone(),
                token: context.amount_to_send.clone(),
                sender: env.contract.address.to_string(),
                receiver: context.receiver.clone(),
                timeout_height: RequestPacketTimeoutHeight {
                    revision_number: None,
                    revision_height: None,
                },
                // 20 minutes should be enough for IBC transfer to go through
                // FIXME: maybe better allow user to set their own timeout?
                timeout_timestamp,
                memo: "".to_string(),
                fee: ibc_fee,
            };

            let mut response = Response::new()
                .add_submessage(SubMsg::reply_always(ibc_transfer, IBC_TRANSFER_REPLY_ID))
                .add_attributes([
                    attr("subaction", "astroport_router_swap"),
                    attr("swapped_amount", fee_balance.amount),
                    attr("subaction", "perform_ibc_transfer"),
                    attr("source_port", context.source_port),
                    attr("source_channel", context.source_channel),
                    attr(
                        "token",
                        format!(
                            "{}{}",
                            context.amount_to_send.amount, context.amount_to_send.denom
                        ),
                    ),
                    attr("sender", env.contract.address.into_string()),
                    attr("receiver", context.receiver),
                    attr("timeout_height", "null"),
                    attr("timeout_timestamp", timeout_timestamp.to_string()),
                ]);

            if let Some(refund) = refund {
                let refund = coin(refund.u128(), context.ibc_fee_denom);
                response = response
                    .add_message(BankMsg::Send {
                        to_address: context.refund_address.into_string(),
                        amount: vec![refund.clone()],
                    })
                    .add_attributes([
                        attr("subaction", "refund_excess_swapped_fee"),
                        attr("amount", format!("{}{}", refund.amount, refund.denom)),
                    ])
            }

            Ok(response)
        }
    }
}

pub(crate) fn reply_ibc_transfer(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    result: SubMsgResult,
) -> ContractResult<Response<NeutronMsg>> {
    // Step 4.
    // Handle immediate reply from IBC transfer module
    // On failure: refund canonical funds and IBC fees back to user
    // On success: store sequence_id and channel to handle IBC callback later

    let context = WRAP_AND_SEND_CONTEXT.load(deps.storage)?;
    let ibc_fee = IBC_FEE.load(deps.storage)?;

    match result {
        // I ignore this error string since I am not sure how to propogate it
        // and inserting it into attributes doesn't sound right at all
        SubMsgResult::Err(_e) => Ok(Response::new()
            .add_message(BankMsg::Send {
                to_address: context.refund_address.into_string(),
                amount: vec![
                    context.amount_to_send,
                    coin(
                        (ibc_fee.ack_fee[0].amount + ibc_fee.timeout_fee[0].amount).u128(),
                        context.ibc_fee_denom,
                    ),
                ],
            })
            .add_attributes([
                attr("action", "cancel_wrap_and_send"),
                attr("reason", "ibc_transfer_failed"),
            ])),
        SubMsgResult::Ok(response) => {
            // IBC transfer module always sets reply data on success:
            // https://github.com/neutron-org/neutron/blob/v1.0.4/x/transfer/keeper/keeper.go#L27-L62
            // hence I can easily use `Option::unwrap()` here
            let data = response.data.unwrap();
            let response: MsgIbcTransferResponse = from_binary(&data)?;
            IBC_TRANSFER_INFO.save(
                deps.storage,
                (response.sequence_id, &response.channel),
                &IbcTransferInfo {
                    refund_address: context.refund_address,
                    ibc_fee,
                    sent_amount: context.amount_to_send,
                },
            )?;
            Ok(Response::new().add_attributes([
                attr("subaction", "ibc_transfer"),
                attr("sequence_id", response.sequence_id.to_string()),
                attr("channel", response.channel),
            ]))
        }
    }
}
