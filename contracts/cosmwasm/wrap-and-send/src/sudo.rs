use crate::{state::IBC_TRANSFER_INFO, ContractResult};
use cosmwasm_std::{attr, BankMsg, Binary, DepsMut, Env, Response};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    sudo::msg::RequestPacket,
};

pub fn sudo_response(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    request: RequestPacket,
    _data: Binary,
) -> ContractResult<Response<NeutronMsg>> {
    let (sequence, source_channel) = extract_sequence_and_channel(request);
    let ibc_transfer_info = IBC_TRANSFER_INFO.load(deps.storage, (sequence, &source_channel))?;

    let refund = &ibc_transfer_info.ibc_fee.timeout_fee[0];
    let refund_msg = BankMsg::Send {
        to_address: ibc_transfer_info.refund_address.into_string(),
        amount: vec![refund.clone()],
    };
    IBC_TRANSFER_INFO.remove(deps.storage, (sequence, &source_channel));

    Ok(Response::new().add_message(refund_msg).add_attributes([
        attr("action", "ibc_ack"),
        attr("status", "success"),
        attr("action", "refund_ibc_timeout_fee"),
        attr("amount", format!("{}{}", refund.amount, refund.denom)),
    ]))
}

pub fn sudo_error(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    request: RequestPacket,
    _details: String,
) -> ContractResult<Response<NeutronMsg>> {
    let (sequence, source_channel) = extract_sequence_and_channel(request);
    let ibc_transfer_info = IBC_TRANSFER_INFO.load(deps.storage, (sequence, &source_channel))?;

    let refund1 = ibc_transfer_info.sent_amount;
    let refund2 = &ibc_transfer_info.ibc_fee.timeout_fee[0];
    let refund_msg = BankMsg::Send {
        to_address: ibc_transfer_info.refund_address.into_string(),
        amount: vec![refund1.clone(), refund2.clone()],
    };
    IBC_TRANSFER_INFO.remove(deps.storage, (sequence, &source_channel));

    Ok(Response::new().add_message(refund_msg).add_attributes([
        attr("action", "ibc_ack"),
        attr("status", "failure"),
        attr("action", "refund_sent_funds"),
        attr("amount", format!("{}{}", refund1.amount, refund1.denom)),
        attr("action", "refund_ibc_timeout_fee"),
        attr("amount", format!("{}{}", refund2.amount, refund2.denom)),
    ]))
}

pub fn sudo_timeout(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    request: RequestPacket,
) -> ContractResult<Response<NeutronMsg>> {
    let (sequence, source_channel) = extract_sequence_and_channel(request);
    let ibc_transfer_info = IBC_TRANSFER_INFO.load(deps.storage, (sequence, &source_channel))?;

    let refund1 = ibc_transfer_info.sent_amount;
    let refund2 = &ibc_transfer_info.ibc_fee.ack_fee[0];
    let refund_msg = BankMsg::Send {
        to_address: ibc_transfer_info.refund_address.into_string(),
        amount: vec![refund1.clone(), refund2.clone()],
    };
    IBC_TRANSFER_INFO.remove(deps.storage, (sequence, &source_channel));

    Ok(Response::new().add_message(refund_msg).add_attributes([
        attr("action", "ibc_timeout"),
        attr("action", "refund_sent_funds"),
        attr("amount", format!("{}{}", refund1.amount, refund1.denom)),
        attr("action", "refund_ibc_ack_fee"),
        attr("amount", format!("{}{}", refund2.amount, refund2.denom)),
    ]))
}

fn extract_sequence_and_channel(request: RequestPacket) -> (u64, String) {
    // we can safely call `Option::unwrap()` since these fields are always set:
    // https://github.com/cosmos/ibc-go/blob/v4.3.1/proto/ibc/core/channel/v1/channel.proto#L97-L104
    (request.sequence.unwrap(), request.source_channel.unwrap())
}
