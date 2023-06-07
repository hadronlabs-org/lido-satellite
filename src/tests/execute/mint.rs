use crate::{
    contract::execute, msg::ExecuteMsg, tests::helpers::instantiate_wrapper, ContractError,
};
use cosmwasm_std::{attr, coin, testing::mock_info, Response, Uint128};
use neutron_sdk::bindings::msg::NeutronMsg;

#[test]
fn no_funds() {
    let (_result, mut deps, env) = instantiate_wrapper("wsteth", "eth", "owner", None);
    let err = execute(
        deps.as_mut(),
        env,
        mock_info("stranger", &[]),
        ExecuteMsg::Mint { receiver: None },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::NothingToMint {});
}

#[test]
fn incorrect_funds() {
    let (_result, mut deps, env) = instantiate_wrapper("wsteth", "eth", "owner", None);
    let err = execute(
        deps.as_mut(),
        env,
        mock_info("stranger", &[coin(10, "ldo")]),
        ExecuteMsg::Mint { receiver: None },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::NothingToMint {});
}

#[test]
fn correct_funds() {
    let (_result, mut deps, env) = instantiate_wrapper("wsteth", "eth", "owner", None);
    let response = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("stranger", &[coin(10, "wsteth")]),
        ExecuteMsg::Mint { receiver: None },
    )
    .unwrap();
    assert_mint_message_and_attrs(&response, env.contract.address, "stranger", 10);
}

#[test]
fn mixed_funds() {
    let (_result, mut deps, env) = instantiate_wrapper("wsteth", "eth", "owner", None);
    let response = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("stranger", &[coin(10, "wsteth"), coin(20, "ldo")]),
        ExecuteMsg::Mint { receiver: None },
    )
    .unwrap();
    assert_mint_message_and_attrs(&response, env.contract.address, "stranger", 10);
}

#[test]
fn with_custom_receiver() {
    let (_result, mut deps, env) = instantiate_wrapper("wsteth", "eth", "owner", None);
    let response = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("stranger", &[coin(11, "wsteth")]),
        ExecuteMsg::Mint {
            receiver: Some("benefitiary".to_string()),
        },
    )
    .unwrap();
    assert_mint_message_and_attrs(&response, env.contract.address, "benefitiary", 11);
}

fn assert_mint_message_and_attrs(
    response: &Response<NeutronMsg>,
    contract_address: impl Into<String>,
    mint_to_address: &str,
    amount: u128,
) {
    assert_eq!(response.messages.len(), 1);
    assert_eq!(
        response.messages[0].msg,
        NeutronMsg::MintTokens {
            denom: format!("factory/{}/eth", contract_address.into()),
            amount: Uint128::new(amount),
            mint_to_address: mint_to_address.to_string(),
        }
        .into()
    );
    assert_eq!(
        response.attributes,
        vec![
            attr("action", "mint"),
            attr("amount", amount.to_string()),
            attr("to", mint_to_address.to_string())
        ]
    )
}
