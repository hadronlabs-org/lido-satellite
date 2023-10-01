use crate::{
    contract::IBC_TRANSFER_REPLY_ID,
    msg::ExecuteMsg,
    state::{IbcTransferInfo, CONFIG, IBC_TRANSFER_CONTEXT},
    ContractError, ContractResult,
};
use astroport::router::{
    ExecuteMsg::ExecuteSwapOperations as AstroportExecuteSwapOperations, SwapOperation,
};
use cosmwasm_std::{
    coin, to_binary, BankMsg, Coin, DepsMut, Env, MessageInfo, Response, SubMsg, Uint128, WasmMsg,
};
use lido_satellite::{
    error::ContractError as LidoSatelliteContractError,
    execute::find_denom,
    msg::{
        ConfigResponse as LidoSatelliteConfigResponse,
        ExecuteMsg::Mint as LidoSatelliteExecuteMint, QueryMsg::Config as LidoSatelliteQueryConfig,
    },
};
use neutron_sdk::{
    bindings::{
        msg::{IbcFee, NeutronMsg},
        query::NeutronQuery,
    },
    query::min_ibc_fee::query_min_ibc_fee,
    sudo::msg::RequestPacketTimeoutHeight,
};
use std::cmp::Ordering;

#[allow(clippy::too_many_arguments)]
pub(crate) fn execute_wrap_and_send(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    source_port: String,
    source_channel: String,
    receiver: String,
    amount_to_swap_for_ibc_fee: Uint128,
    ibc_fee_denom: String,
    astroport_swap_operations: Vec<SwapOperation>,
    refund_address: String,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let lido_satellite_config: LidoSatelliteConfigResponse = deps
        .querier
        .query_wasm_smart(&config.lido_satellite, &LidoSatelliteQueryConfig {})?;

    let received_amount = find_denom(&info.funds, &lido_satellite_config.bridged_denom)?
        .ok_or(LidoSatelliteContractError::NothingToMint {})?
        .amount;
    let amount_to_send = coin(
        received_amount
            .checked_sub(amount_to_swap_for_ibc_fee)?
            .u128(),
        &lido_satellite_config.canonical_denom,
    );
    let amount_to_swap_for_ibc_fee = coin(
        amount_to_swap_for_ibc_fee.u128(),
        &lido_satellite_config.canonical_denom,
    );

    // TODO: make it a method with a validation
    let min_ibc_fee = {
        let mut fee = query_min_ibc_fee(deps.as_ref())?.min_fee;
        // fee.recv_fee is always empty
        fee.ack_fee.retain(|coin| coin.denom == ibc_fee_denom);
        fee.timeout_fee.retain(|coin| coin.denom == ibc_fee_denom);
        fee
    };

    let wrap_msg = WasmMsg::Execute {
        contract_addr: config.lido_satellite.into_string(),
        msg: to_binary(&LidoSatelliteExecuteMint { receiver: None })?,
        funds: info.funds,
    };
    let swap_msg = WasmMsg::Execute {
        contract_addr: config.astroport_router.into_string(),
        msg: to_binary(&AstroportExecuteSwapOperations {
            operations: astroport_swap_operations,
            minimum_receive: Some(
                min_ibc_fee.ack_fee[0].amount + min_ibc_fee.timeout_fee[0].amount,
            ),
            to: None,
            max_spread: None,
        })?,
        funds: vec![amount_to_swap_for_ibc_fee],
    };
    let callback_msg = WasmMsg::Execute {
        contract_addr: env.contract.address.into_string(),
        msg: to_binary(&ExecuteMsg::SwapCallback {
            source_port,
            source_channel,
            receiver,
            amount_to_send,
            min_ibc_fee,
            refund_address,
        })?,
        funds: vec![],
    };

    Ok(Response::new().add_messages([wrap_msg, swap_msg, callback_msg]))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn execute_swap_callback(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    source_port: String,
    source_channel: String,
    receiver: String,
    amount_to_send: Coin,
    min_ibc_fee: IbcFee,
    refund_address: String,
) -> ContractResult<Response<NeutronMsg>> {
    if info.sender != env.contract.address {
        return Err(ContractError::InternalMethod {});
    }

    let refund_address = deps.api.addr_validate(&refund_address)?;

    let total_ibc_fee = min_ibc_fee.ack_fee[0].amount + min_ibc_fee.timeout_fee[0].amount;
    let ibc_fee_denom = min_ibc_fee.ack_fee[0].denom.clone();
    let fee_balance = deps
        .querier
        .query_balance(&env.contract.address, &ibc_fee_denom)?;
    let refund = match fee_balance.amount.cmp(&total_ibc_fee) {
        Ordering::Less => {
            // should never happen, but let's be cautious
            return Err(ContractError::SwappedForLessThanRequested {});
        }
        Ordering::Equal => None,
        Ordering::Greater => Some(fee_balance.amount - total_ibc_fee),
    };

    IBC_TRANSFER_CONTEXT.save(
        deps.storage,
        &IbcTransferInfo {
            refund_address: refund_address.clone(),
            ibc_fee: min_ibc_fee.clone(),
            sent_amount: amount_to_send.clone(),
        },
    )?;

    // 20 minutes should be enough for IBC transfer to go through
    // FIXME: maybe better allow user to set their own timeout?
    let timeout_timestamp = env.block.time.plus_minutes(20).nanos();
    let ibc_transfer = NeutronMsg::IbcTransfer {
        source_port,
        source_channel,
        token: amount_to_send,
        sender: env.contract.address.into_string(),
        receiver,
        timeout_height: RequestPacketTimeoutHeight {
            revision_number: None,
            revision_height: None,
        },
        timeout_timestamp,
        memo: "".to_string(),
        fee: min_ibc_fee,
    };

    let mut response = Response::new().add_submessage(SubMsg::reply_on_success(
        ibc_transfer,
        IBC_TRANSFER_REPLY_ID,
    ));

    if let Some(refund) = refund {
        let refund = coin(refund.u128(), ibc_fee_denom);
        response = response.add_message(BankMsg::Send {
            to_address: refund_address.into_string(),
            amount: vec![refund.clone()],
        })
    }

    Ok(response)
}
