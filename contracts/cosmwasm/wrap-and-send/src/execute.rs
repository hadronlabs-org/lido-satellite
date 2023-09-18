use crate::{state::CONFIG, ContractResult};
use cosmwasm_std::{
    attr, coin, to_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, WasmMsg,
};
use lido_satellite::{
    error::ContractError as LidoSatelliteContractError, execute::find_denom,
    msg::ConfigResponse as LidoSatelliteConfigResponse,
    msg::ExecuteMsg::Mint as LidoSatelliteExecuteMint,
    msg::QueryMsg::Config as LidoSatelliteQueryConfig,
};
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    query::min_ibc_fee::query_min_ibc_fee,
    sudo::msg::RequestPacketTimeoutHeight,
};

pub(crate) fn execute_wrap_and_send(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    source_port: String,
    source_channel: String,
    receiver: String,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let lido_satellite_config: LidoSatelliteConfigResponse = deps
        .querier
        .query_wasm_smart(&config.lido_satellite, &LidoSatelliteQueryConfig {})?;

    // Lido Satellite will filter funds, however, we have to filter them ourselves anyway,
    // because we need to know the amount of funds to send within next IBC message
    let amount_to_send = find_denom(&info.funds, &lido_satellite_config.bridged_denom)?
        .ok_or(LidoSatelliteContractError::NothingToMint {})?
        .amount
        .u128();
    let ibc_fee = {
        let mut fee = query_min_ibc_fee(deps.as_ref())?.min_fee;
        // fee.recv_fee is always empty
        fee.ack_fee
            .retain(|coin| coin.denom == config.ibc_fee_denom);
        fee.timeout_fee
            .retain(|coin| coin.denom == config.ibc_fee_denom);
        fee
    };

    let wrap_msg: CosmosMsg<NeutronMsg> = WasmMsg::Execute {
        contract_addr: config.lido_satellite.into_string(),
        msg: to_binary(&LidoSatelliteExecuteMint { receiver: None })?,
        funds: info.funds,
    }
    .into();
    let ibc_msg: CosmosMsg<NeutronMsg> = NeutronMsg::IbcTransfer {
        source_port: source_port.clone(),
        source_channel: source_channel.clone(),
        token: coin(amount_to_send, &lido_satellite_config.canonical_denom),
        sender: env.contract.address.into_string(),
        receiver: receiver.clone(),
        timeout_height: RequestPacketTimeoutHeight {
            revision_number: None,
            revision_height: None,
        },
        timeout_timestamp: env.block.time.plus_hours(1).nanos(),
        memo: "".to_string(),
        fee: ibc_fee,
    }
    .into();

    Ok(Response::new()
        .add_messages([wrap_msg, ibc_msg])
        .add_attributes([
            attr("action", "wrap_and_send"),
            attr("source_denom", lido_satellite_config.bridged_denom),
            attr("target_denom", lido_satellite_config.canonical_denom),
            attr("amount", amount_to_send.to_string()),
            attr("sender", info.sender),
            attr("receiver", receiver),
            attr("source_port", source_port),
            attr("source_channel", source_channel),
        ]))
}
