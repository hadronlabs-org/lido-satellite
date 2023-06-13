use crate::{
    contract::execute, msg::ExecuteMsg, state::CONFIG, tests::helpers::instantiate_wrapper,
    ContractError,
};
use cosmwasm_std::{attr, coin, testing::mock_info, BankMsg, Response, Uint128};
use neutron_sdk::bindings::msg::NeutronMsg;

#[test]
fn no_funds() {
    let (_result, mut deps, env) = instantiate_wrapper("wsteth", "eth");
    let err = execute(
        deps.as_mut(),
        env,
        mock_info("stranger", &[]),
        ExecuteMsg::Burn { receiver: None },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::NothingToBurn {});
}

#[test]
fn incorrect_funds() {
    let (_result, mut deps, env) = instantiate_wrapper("wsteth", "eth");
    let err = execute(
        deps.as_mut(),
        env,
        mock_info("stranger", &[coin(10, "ldo")]),
        ExecuteMsg::Burn { receiver: None },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::NothingToBurn {});
}

#[test]
fn correct_funds() {
    let (_result, mut deps, env) = instantiate_wrapper("wsteth", "eth");
    let config = CONFIG.load(deps.as_mut().storage).unwrap();
    let full_tokenfactory_denom = config.get_full_tokenfactory_denom(&env.contract.address);
    let response = execute(
        deps.as_mut(),
        env,
        mock_info("stranger", &[coin(10, &full_tokenfactory_denom)]),
        ExecuteMsg::Burn { receiver: None },
    )
    .unwrap();

    assert_burn_send_messages_and_attrs(
        &response,
        "stranger",
        10,
        full_tokenfactory_denom,
        "wsteth",
    );
}

#[test]
fn mixed_funds() {
    let (_result, mut deps, env) = instantiate_wrapper("wsteth", "eth");
    let config = CONFIG.load(deps.as_mut().storage).unwrap();
    let full_tokenfactory_denom = config.get_full_tokenfactory_denom(&env.contract.address);
    let response = execute(
        deps.as_mut(),
        env,
        mock_info(
            "stranger",
            &[coin(10, &full_tokenfactory_denom), coin(20, "ldo")],
        ),
        ExecuteMsg::Burn { receiver: None },
    )
    .unwrap();

    assert_burn_send_messages_and_attrs(
        &response,
        "stranger",
        10,
        full_tokenfactory_denom,
        "wsteth",
    );
}

#[test]
fn with_custom_receiver() {
    let (_result, mut deps, env) = instantiate_wrapper("wsteth", "eth");
    let config = CONFIG.load(deps.as_mut().storage).unwrap();
    let full_tokenfactory_denom = config.get_full_tokenfactory_denom(&env.contract.address);
    let response = execute(
        deps.as_mut(),
        env,
        mock_info("stranger", &[coin(12, &full_tokenfactory_denom)]),
        ExecuteMsg::Burn {
            receiver: Some("benefitiary".to_string()),
        },
    )
    .unwrap();

    assert_burn_send_messages_and_attrs(
        &response,
        "benefitiary",
        12,
        full_tokenfactory_denom,
        "wsteth",
    );
}

fn assert_burn_send_messages_and_attrs(
    response: &Response<NeutronMsg>,
    receiver: &str,
    amount: u128,
    canonical_subdenom: impl Into<String>,
    bridged_denom: impl Into<String>,
) {
    assert_eq!(response.messages.len(), 2);
    assert_eq!(
        response.messages[0].msg,
        NeutronMsg::BurnTokens {
            denom: canonical_subdenom.into(),
            amount: Uint128::new(amount),
            burn_from_address: "".to_string(),
        }
        .into()
    );
    assert_eq!(
        response.messages[1].msg,
        BankMsg::Send {
            to_address: receiver.to_string(),
            amount: vec![coin(amount, bridged_denom)]
        }
        .into()
    );
    assert_eq!(
        response.attributes,
        vec![
            attr("action", "burn"),
            attr("amount", amount.to_string()),
            attr("from", receiver)
        ]
    )
}
