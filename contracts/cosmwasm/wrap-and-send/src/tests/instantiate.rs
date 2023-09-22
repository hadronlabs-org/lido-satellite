use crate::tests::helpers::{assert_config, instantiate_wrapper};
use cosmwasm_std::{attr, testing::MockQuerier, Response};
use neutron_sdk::bindings::msg::NeutronMsg;

#[test]
fn success() {
    let (result, deps, _env) =
        instantiate_wrapper::<MockQuerier>("lido_satellite", "astroport_router");
    let response = result.unwrap();
    assert_instantiate_response(&response, "lido_satellite", "astroport_router");
    assert_config(deps.as_ref(), "lido_satellite", "astroport_router");
}

fn assert_instantiate_response(
    response: &Response<NeutronMsg>,
    lido_satellite: &str,
    astroport_router: &str,
) {
    assert!(response.messages.is_empty());
    assert_eq!(
        response.attributes,
        vec![
            attr("action", "instantiate"),
            attr("lido_satellite", lido_satellite),
            attr("astroport_router", astroport_router)
        ]
    );
}
