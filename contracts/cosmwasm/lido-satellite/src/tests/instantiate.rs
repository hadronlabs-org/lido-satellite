use crate::{
    tests::helpers::{assert_config, instantiate_wrapper, VALID_IBC_DENOM},
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
fn too_short_bridged_denom() {
    let (result, _deps, _env) = instantiate_wrapper(
        // misses one character in the end
        "ibc/584A4A23736884E0C198FD1EE932455A9357A492A7B94324E4A02B562868783",
        "subdenom",
    );
    let err = result.unwrap_err();
    assert_eq!(
        err,
        ContractError::InvalidIbcDenom {
            denom: "ibc/584A4A23736884E0C198FD1EE932455A9357A492A7B94324E4A02B562868783"
                .to_string(),
            reason: "expected length of 68 chars".to_string(),
        }
    );
}

#[test]
fn too_long_bridged_denom() {
    let (result, _deps, _env) = instantiate_wrapper(
        // one extra character in the end
        "ibc/584A4A23736884E0C198FD1EE932455A9357A492A7B94324E4A02B56286878312",
        "subdenom",
    );
    let err = result.unwrap_err();
    assert_eq!(
        err,
        ContractError::InvalidIbcDenom {
            denom: "ibc/584A4A23736884E0C198FD1EE932455A9357A492A7B94324E4A02B56286878312"
                .to_string(),
            reason: "expected length of 68 chars".to_string(),
        }
    );
}

#[test]
fn bridged_denom_with_invalid_prefix() {
    let (result, _deps, _env) = instantiate_wrapper(
        // "ebc/" instead of "ibc/"
        "ebc/584A4A23736884E0C198FD1EE932455A9357A492A7B94324E4A02B5628687831",
        "subdenom",
    );
    let err = result.unwrap_err();
    assert_eq!(
        err,
        ContractError::InvalidIbcDenom {
            denom: "ebc/584A4A23736884E0C198FD1EE932455A9357A492A7B94324E4A02B5628687831"
                .to_string(),
            reason: "expected prefix 'ibc/'".to_string(),
        }
    );
}

#[test]
fn bridged_denom_with_invalid_hash() {
    let (result, _deps, _env) = instantiate_wrapper(
        // first character of hexadecimal hash is X
        "ibc/X84A4A23736884E0C198FD1EE932455A9357A492A7B94324E4A02B5628687831",
        "subdenom",
    );
    let err = result.unwrap_err();
    assert_eq!(
        err,
        ContractError::InvalidIbcDenom {
            denom: "ibc/X84A4A23736884E0C198FD1EE932455A9357A492A7B94324E4A02B5628687831"
                .to_string(),
            reason: "invalid denom hash".to_string(),
        }
    );
}

#[test]
fn bridged_denom_with_invalid_hash_lowercase() {
    let (result, _deps, _env) = instantiate_wrapper(
        // first character of hexadecimal hash is lowercase c, should be uppercase C
        "ibc/c84A4A23736884E0C198FD1EE932455A9357A492A7B94324E4A02B5628687831",
        "subdenom",
    );
    let err = result.unwrap_err();
    assert_eq!(
        err,
        ContractError::InvalidIbcDenom {
            denom: "ibc/c84A4A23736884E0C198FD1EE932455A9357A492A7B94324E4A02B5628687831"
                .to_string(),
            reason: "invalid denom hash".to_string(),
        }
    );
}

#[test]
fn empty_canonical_subdenom() {
    let (result, _deps, _env) = instantiate_wrapper(VALID_IBC_DENOM, "");
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
    let (result, deps, _env) = instantiate_wrapper(VALID_IBC_DENOM, "subdenom");
    let response = result.unwrap();
    assert_create_denom_msg_and_attrs(&response, VALID_IBC_DENOM, "subdenom");
    assert_config(deps.as_ref(), VALID_IBC_DENOM, "subdenom");
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
