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

    let wsteth_fund =
        find_denom(&info.funds, &config.wsteth_denom).ok_or(ContractError::NothingToMint {})?;

    let mint_msg: CosmosMsg<NeutronMsg> =
        NeutronMsg::submit_mint_tokens(full_tokenfactory_denom, wsteth_fund.amount, &receiver)
            .into();

    Ok(Response::new().add_message(mint_msg).add_attributes([
        attr("action", "mint"),
        attr("amount", wsteth_fund.amount),
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

    let amount_to_burn = find_denom(&info.funds, &full_tokenfactory_denom)
        .ok_or(ContractError::NothingToBurn {})?
        .amount;

    let burn_msg: CosmosMsg<NeutronMsg> =
        NeutronMsg::submit_burn_tokens(full_tokenfactory_denom, amount_to_burn).into();
    let send_msg = BankMsg::Send {
        to_address: receiver.to_string(),
        amount: vec![coin(amount_to_burn.u128(), config.wsteth_denom)],
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

pub(crate) fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    wsteth_denom: Option<String>,
    subdenom: Option<String>,
    owner: Option<String>,
) -> ContractResult<Response<NeutronMsg>> {
    let mut config = CONFIG.load(deps.storage)?;
    let mut attributes = vec![attr("action", "update_config")];

    if config.owner != info.sender {
        return Err(ContractError::Unautorized {});
    }

    if let Some(wsteth_denom) = wsteth_denom {
        config.wsteth_denom = wsteth_denom;
        attributes.push(attr("new_wsteth_denom", &config.wsteth_denom));
    }
    if let Some(subdenom) = subdenom {
        config.subdenom = subdenom;
        attributes.push(attr("new_subdenom", &config.subdenom));
    }
    if let Some(owner) = owner {
        config.owner = deps.api.addr_validate(&owner)?;
        attributes.push(attr("new_owner", &config.owner));
    }

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attributes(attributes))
}

fn find_denom<'a>(funds: &'a [Coin], target_denom: &str) -> Option<&'a Coin> {
    funds.iter().find(|fund| fund.denom == target_denom)
}
