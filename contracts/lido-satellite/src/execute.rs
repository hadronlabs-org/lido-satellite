use crate::{state::CONFIG, ContractError, ContractResult};
use cosmwasm_std::{attr, coin, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response};
use neutron_sdk::bindings::msg::NeutronMsg;

pub(crate) fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    receiver: Option<String>,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let full_tokenfactory_denom = config.get_full_tokenfactory_denom(env.contract.address);
    let receiver = receiver.map_or(Ok(info.sender), |addr| deps.api.addr_validate(&addr))?;

    let bridged_funds =
        find_denom(&info.funds, &config.bridged_denom)?.ok_or(ContractError::NothingToMint {})?;

    let mint_msg: CosmosMsg<NeutronMsg> =
        NeutronMsg::submit_mint_tokens(full_tokenfactory_denom, bridged_funds.amount, &receiver)
            .into();

    Ok(Response::new().add_message(mint_msg).add_attributes([
        attr("action", "mint"),
        attr("amount", bridged_funds.amount),
        attr("to", receiver),
    ]))
}

pub(crate) fn execute_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    receiver: Option<String>,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    let full_tokenfactory_denom = config.get_full_tokenfactory_denom(env.contract.address);
    let receiver = receiver.map_or(Ok(info.sender), |addr| deps.api.addr_validate(&addr))?;

    let amount_to_burn = find_denom(&info.funds, &full_tokenfactory_denom)?
        .ok_or(ContractError::NothingToBurn {})?
        .amount;

    let burn_msg: CosmosMsg<NeutronMsg> =
        NeutronMsg::submit_burn_tokens(full_tokenfactory_denom, amount_to_burn).into();
    let send_msg = BankMsg::Send {
        to_address: receiver.to_string(),
        amount: vec![coin(amount_to_burn.u128(), config.bridged_denom)],
    }
    .into();

    Ok(Response::new()
        .add_messages([burn_msg, send_msg])
        .add_attributes([
            attr("action", "burn"),
            attr("amount", amount_to_burn),
            attr("from", receiver),
        ]))
}

fn find_denom<'a>(funds: &'a [Coin], target_denom: &str) -> ContractResult<Option<&'a Coin>> {
    match funds.len() {
        0 => Ok(None),
        1 => Ok({
            let fund = &funds[0];
            if fund.denom == target_denom {
                Some(fund)
            } else {
                None
            }
        }),
        _ => Err(ContractError::ExtraFunds {}),
    }
}
