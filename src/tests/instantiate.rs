use crate::{
    tests::helpers::{assert_config, instantiate_wrapper},
    ContractError,
};
use cosmwasm_std::{attr, Response};
use neutron_sdk::bindings::msg::NeutronMsg;

#[test]
fn empty_denoms() {
    let (result, _deps, _env) = instantiate_wrapper("", "", "owner", None);
    let err = result.unwrap_err();
    assert!(matches!(err, ContractError::EmptyDenom { .. }))
}

#[test]
fn empty_wsteth_denom() {
    let (result, _deps, _env) = instantiate_wrapper("", "subdenom", "owner", None);
    let err = result.unwrap_err();
    assert_eq!(
        err,
        ContractError::EmptyDenom {
            kind: "wsteth_denom".to_string()
        }
    );
}

#[test]
fn empty_subdenom() {
    let (result, _deps, _env) = instantiate_wrapper("wsteth", "", "owner", None);
    let err = result.unwrap_err();
    assert_eq!(
        err,
        ContractError::EmptyDenom {
            kind: "subdenom".to_string()
        }
    );
}

#[test]
fn without_owner() {
    let (result, deps, _env) = instantiate_wrapper("wsteth", "subdenom", "owner", None);
    let response = result.unwrap();
    assert_create_denom_msg_and_attrs(&response, "wsteth", "subdenom", "owner");
    assert_config(deps.as_ref(), "wsteth", "subdenom", "owner");
}

#[test]
fn with_explicit_owner() {
    let (result, deps, _env) = instantiate_wrapper("wsteth", "subdenom", "owner", Some("multisig"));
    let response = result.unwrap();
    assert_create_denom_msg_and_attrs(&response, "wsteth", "subdenom", "multisig");
    assert_config(deps.as_ref(), "wsteth", "subdenom", "multisig");
}

fn assert_create_denom_msg_and_attrs(
    response: &Response<NeutronMsg>,
    wsteth_denom: &str,
    subdenom: &str,
    owner: &str,
) {
    assert_eq!(response.messages.len(), 1);
    assert_eq!(
        response.messages[0].msg,
        NeutronMsg::CreateDenom {
            subdenom: subdenom.to_string()
        }
        .into()
    );
    assert_eq!(
        response.attributes,
        vec![
            attr("wsteth_denom", wsteth_denom),
            attr("subdenom", subdenom),
            attr("owner", owner)
        ]
    );
}
