use crate::{
    contract::execute,
    msg::ExecuteMsg,
    tests::helpers::{instantiate_wrapper, VALID_IBC_DENOM},
    ContractError,
};
use cosmwasm_std::{attr, coin, testing::mock_info, Response, Uint128};
use neutron_sdk::bindings::msg::NeutronMsg;

#[test]
fn no_funds() {
    let (_result, mut deps, env) = instantiate_wrapper(VALID_IBC_DENOM, "eth");
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
    let (_result, mut deps, env) = instantiate_wrapper(VALID_IBC_DENOM, "eth");
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
    let (_result, mut deps, env) = instantiate_wrapper(VALID_IBC_DENOM, "eth");
    let response = execute(
        deps.as_mut(),
        env,
        mock_info("stranger", &[coin(10, VALID_IBC_DENOM)]),
        ExecuteMsg::Mint { receiver: None },
    )
    .unwrap();
    assert_mint_message_and_attrs(&response, "stranger", "stranger", 10, "eth");
}

#[test]
fn mixed_funds() {
    let (_result, mut deps, env) = instantiate_wrapper(VALID_IBC_DENOM, "eth");
    let err = execute(
        deps.as_mut(),
        env,
        mock_info("stranger", &[coin(10, VALID_IBC_DENOM), coin(20, "ldo")]),
        ExecuteMsg::Mint { receiver: None },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::ExtraFunds {});
}

#[test]
fn with_custom_receiver() {
    let (_result, mut deps, env) = instantiate_wrapper(VALID_IBC_DENOM, "eth");
    let response = execute(
        deps.as_mut(),
        env,
        mock_info("stranger", &[coin(11, VALID_IBC_DENOM)]),
        ExecuteMsg::Mint {
            receiver: Some("benefitiary".to_string()),
        },
    )
    .unwrap();
    assert_mint_message_and_attrs(&response, "stranger", "benefitiary", 11, "eth");
}

fn assert_mint_message_and_attrs(
    response: &Response<NeutronMsg>,
    sender: &str,
    mint_to_address: &str,
    amount: u128,
    canonical_denom: impl Into<String>,
) {
    assert_eq!(response.messages.len(), 1);
    assert_eq!(
        response.messages[0].msg,
        NeutronMsg::MintTokens {
            denom: canonical_denom.into(),
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
            attr("sender", sender),
            attr("receiver", mint_to_address)
        ]
    )
}
