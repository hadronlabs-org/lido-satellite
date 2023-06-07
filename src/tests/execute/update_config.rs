use crate::{
    contract::execute,
    msg::ExecuteMsg,
    tests::helpers::{assert_config, instantiate_wrapper},
    ContractError,
};
use cosmwasm_std::{attr, testing::mock_info};

#[test]
fn unauthorized() {
    let (_result, mut deps, env) = instantiate_wrapper("wsteth", "eth", "owner", None);
    let err = execute(
        deps.as_mut(),
        env,
        mock_info("stranger", &[]),
        ExecuteMsg::UpdateConfig {
            wsteth_denom: None,
            subdenom: None,
            owner: None,
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unautorized {});
}

#[test]
fn only_wsteth_denom() {
    let (_result, mut deps, env) = instantiate_wrapper("wsteth", "eth", "owner", None);
    let response = execute(
        deps.as_mut(),
        env,
        mock_info("owner", &[]),
        ExecuteMsg::UpdateConfig {
            wsteth_denom: Some("new_wsteth".to_string()),
            subdenom: None,
            owner: None,
        },
    )
    .unwrap();

    assert_eq!(response.messages.len(), 0);
    assert_eq!(
        response.attributes,
        vec![
            attr("action", "update_config"),
            attr("new_wsteth_denom", "new_wsteth")
        ]
    );
    assert_config(deps.as_ref(), "new_wsteth", "eth", "owner");
}

#[test]
fn only_subdenom() {
    let (_result, mut deps, env) = instantiate_wrapper("wsteth", "eth", "owner", None);
    let response = execute(
        deps.as_mut(),
        env,
        mock_info("owner", &[]),
        ExecuteMsg::UpdateConfig {
            wsteth_denom: None,
            subdenom: Some("new_eth".to_string()),
            owner: None,
        },
    )
    .unwrap();

    assert_eq!(response.messages.len(), 0);
    assert_eq!(
        response.attributes,
        vec![
            attr("action", "update_config"),
            attr("new_subdenom", "new_eth")
        ]
    );
    assert_config(deps.as_ref(), "wsteth", "new_eth", "owner");
}

#[test]
fn only_owner() {
    let (_result, mut deps, env) = instantiate_wrapper("wsteth", "eth", "owner", None);
    let response = execute(
        deps.as_mut(),
        env,
        mock_info("owner", &[]),
        ExecuteMsg::UpdateConfig {
            wsteth_denom: None,
            subdenom: None,
            owner: Some("new_owner".to_string()),
        },
    )
    .unwrap();

    assert_eq!(response.messages.len(), 0);
    assert_eq!(
        response.attributes,
        vec![
            attr("action", "update_config"),
            attr("new_owner", "new_owner")
        ]
    );
    assert_config(deps.as_ref(), "wsteth", "eth", "new_owner");
}

#[test]
fn all_fields() {
    let (_result, mut deps, env) = instantiate_wrapper("wsteth", "eth", "owner", None);
    let response = execute(
        deps.as_mut(),
        env,
        mock_info("owner", &[]),
        ExecuteMsg::UpdateConfig {
            wsteth_denom: Some("new_wsteth".to_string()),
            subdenom: Some("new_eth".to_string()),
            owner: Some("new_owner".to_string()),
        },
    )
    .unwrap();

    assert_eq!(response.messages.len(), 0);
    assert_eq!(
        response.attributes,
        vec![
            attr("action", "update_config"),
            attr("new_wsteth_denom", "new_wsteth"),
            attr("new_subdenom", "new_eth"),
            attr("new_owner", "new_owner")
        ]
    );
    assert_config(deps.as_ref(), "new_wsteth", "new_eth", "new_owner");
}

#[test]
fn no_fields() {
    let (_result, mut deps, env) = instantiate_wrapper("wsteth", "eth", "owner", None);
    let response = execute(
        deps.as_mut(),
        env,
        mock_info("owner", &[]),
        ExecuteMsg::UpdateConfig {
            wsteth_denom: None,
            subdenom: None,
            owner: None,
        },
    )
    .unwrap();

    assert_eq!(response.messages.len(), 0);
    assert_eq!(response.attributes, vec![attr("action", "update_config"),]);
    assert_config(deps.as_ref(), "wsteth", "eth", "owner");
}
