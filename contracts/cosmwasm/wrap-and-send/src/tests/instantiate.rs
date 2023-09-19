use crate::tests::helpers::{assert_config, instantiate_wrapper};
use cosmwasm_std::{attr, Response};
use neutron_sdk::bindings::msg::NeutronMsg;

#[test]
fn success_no_owner() {
    let (result, deps, _env) = instantiate_wrapper("lido_satellite", "untrn", None);
    let response = result.unwrap();
    assert_instantiate_response(&response, "lido_satellite", "untrn", None);
    assert_config(deps.as_ref(), "lido_satellite", "untrn", None);
}

#[test]
fn success_with_owner() {
    let (result, deps, _env) = instantiate_wrapper("lido_satellite", "untrn", Some("owner"));
    let response = result.unwrap();
    assert_instantiate_response(&response, "lido_satellite", "untrn", Some("owner"));
    assert_config(deps.as_ref(), "lido_satellite", "untrn", Some("owner"));
}

fn assert_instantiate_response(
    response: &Response<NeutronMsg>,
    lido_satellite: &str,
    ibc_fee_denom: &str,
    owner: Option<&str>,
) {
    assert!(response.messages.is_empty());
    assert_eq!(
        response.attributes.len(),
        if owner.is_some() { 3 } else { 2 }
    );

    assert_eq!(
        response.attributes[0],
        attr("lido_satellite", lido_satellite)
    );
    assert_eq!(response.attributes[1], attr("ibc_fee_denom", ibc_fee_denom));
    if let Some(owner) = owner {
        assert_eq!(response.attributes[2], attr("owner", owner));
    }
}
