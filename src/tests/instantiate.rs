use crate::{
    tests::helpers::{assert_config, instantiate_wrapper},
    ContractError,
};
use cosmwasm_std::{attr, Response};
use neutron_sdk::bindings::msg::NeutronMsg;

#[test]
fn empty_denoms() {
    let (result, _deps, _env) = instantiate_wrapper("", "");
    let err = result.unwrap_err();
    assert!(matches!(err, ContractError::EmptyDenom { .. }))
}

#[test]
fn empty_bridged_denom() {
    let (result, _deps, _env) = instantiate_wrapper("", "subdenom");
    let err = result.unwrap_err();
    assert_eq!(
        err,
        ContractError::EmptyDenom {
            kind: "bridged_denom".to_string()
        }
    );
}

#[test]
fn empty_canonical_subdenom() {
    let (result, _deps, _env) = instantiate_wrapper("wsteth", "");
    let err = result.unwrap_err();
    assert_eq!(
        err,
        ContractError::EmptyDenom {
            kind: "canonical_subdenom".to_string()
        }
    );
}

#[test]
fn success() {
    let (result, deps, _env) = instantiate_wrapper("wsteth", "subdenom");
    let response = result.unwrap();
    assert_create_denom_msg_and_attrs(&response, "wsteth", "subdenom");
    assert_config(deps.as_ref(), "wsteth", "subdenom");
}

fn assert_create_denom_msg_and_attrs(
    response: &Response<NeutronMsg>,
    bridged_denom: &str,
    canonical_subdenom: &str,
) {
    assert_eq!(response.messages.len(), 1);
    assert_eq!(
        response.messages[0].msg,
        NeutronMsg::CreateDenom {
            subdenom: canonical_subdenom.to_string()
        }
        .into()
    );
    assert_eq!(
        response.attributes,
        vec![
            attr("bridged_denom", bridged_denom),
            attr("canonical_subdenom", canonical_subdenom),
        ]
    );
}
