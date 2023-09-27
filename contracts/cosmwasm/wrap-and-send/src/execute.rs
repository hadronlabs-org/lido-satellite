use crate::{
    contract::LIDO_SATELLITE_WRAP_REPLY_ID,
    state::{WrapAndSendContext, CONFIG, WRAP_AND_SEND_CONTEXT},
    ContractResult,
};
use astroport::router::SwapOperation;
use cosmwasm_std::{
    attr, coin, to_binary, BankMsg, DepsMut, Env, MessageInfo, Response, SubMsg, Uint128, WasmMsg,
};
use lido_satellite::{
    error::ContractError as LidoSatelliteContractError,
    execute::find_denom,
    msg::{
        ConfigResponse as LidoSatelliteConfigResponse,
        ExecuteMsg::Mint as LidoSatelliteExecuteMint, QueryMsg::Config as LidoSatelliteQueryConfig,
    },
};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};

#[allow(clippy::too_many_arguments)]
pub(crate) fn execute_wrap_and_send(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    info: MessageInfo,
    source_port: String,
    source_channel: String,
    receiver: String,
    amount_to_swap_for_ibc_fee: Uint128,
    ibc_fee_denom: String,
    astroport_swap_operations: Vec<SwapOperation>,
    refund_address: String,
) -> ContractResult<Response<NeutronMsg>> {
    // Step 1.
    // Issue wrap message to Lido Satellite and handle reply

    let config = CONFIG.load(deps.storage)?;
    let lido_satellite_config: LidoSatelliteConfigResponse = deps
        .querier
        .query_wasm_smart(&config.lido_satellite, &LidoSatelliteQueryConfig {})?;

    let received_amount = find_denom(&info.funds, &lido_satellite_config.bridged_denom)?
        // it is okay to return an error here, since we can be sure that
        // Axelar will not supply more than one denom, it is impossible to do so
        // using current implementation of IBC Transfer module
        .ok_or(LidoSatelliteContractError::NothingToMint {})?
        .amount;
    let amount_to_send = match received_amount.checked_sub(amount_to_swap_for_ibc_fee) {
        Err(_) => {
            return Ok(Response::new()
                .add_message(BankMsg::Send {
                    to_address: refund_address.to_string(),
                    amount: info.funds,
                })
                .add_attributes([
                    attr("action", "cancel_wrap_and_send"),
                    attr("reason", "not_enough_funds_to_swap"),
                    attr(
                        "provided",
                        format!("{}{}", received_amount, lido_satellite_config.bridged_denom),
                    ),
                    attr(
                        "required",
                        format!(
                            "{}{}",
                            amount_to_swap_for_ibc_fee, lido_satellite_config.bridged_denom
                        ),
                    ),
                ]))
        }
        Ok(v) => v,
    };

    let wrap_msg = WasmMsg::Execute {
        contract_addr: config.lido_satellite.into_string(),
        msg: to_binary(&LidoSatelliteExecuteMint { receiver: None })?,
        funds: info.funds,
    };

    let refund_address = deps.api.addr_validate(&refund_address)?;
    let amount_to_wrap = coin(received_amount.u128(), &lido_satellite_config.bridged_denom);
    let amount_to_send = coin(
        amount_to_send.u128(),
        &lido_satellite_config.canonical_denom,
    );
    let amount_to_swap_for_ibc_fee = coin(
        amount_to_swap_for_ibc_fee.u128(),
        &lido_satellite_config.canonical_denom,
    );
    WRAP_AND_SEND_CONTEXT.save(
        deps.storage,
        &WrapAndSendContext {
            source_port,
            source_channel,
            receiver,
            astroport_swap_operations,
            refund_address,
            amount_to_wrap,
            amount_to_send,
            amount_to_swap_for_ibc_fee,
            ibc_fee_denom,
        },
    )?;

    Ok(Response::new()
        .add_submessage(SubMsg::reply_always(wrap_msg, LIDO_SATELLITE_WRAP_REPLY_ID))
        .add_attributes([attr("action", "wrap_and_send")]))
}
