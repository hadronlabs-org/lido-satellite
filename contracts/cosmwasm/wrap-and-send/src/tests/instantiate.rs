use crate::{
    contract::instantiate,
    msg::InstantiateMsg,
    state::{Config, CONFIG},
    tests::helpers::bin_request_to_query_request,
};
use cosmwasm_std::{
    attr,
    testing::{mock_env, mock_info, MockApi, MockStorage},
    to_binary, Addr, ContractResult, OwnedDeps, Querier, QuerierResult, QueryRequest, SystemResult,
    WasmQuery,
};
use lido_satellite::msg::{
    ConfigResponse as LidoSatelliteQueryConfigResponse,
    QueryMsg::Config as LidoSatelliteQueryConfig,
};
use neutron_sdk::bindings::query::NeutronQuery;
use std::marker::PhantomData;

#[derive(Default)]
struct CustomMockQuerier {}

impl Querier for CustomMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request = match bin_request_to_query_request::<NeutronQuery>(bin_request) {
            Ok(v) => v,
            Err(e) => return e,
        };
        match request {
            QueryRequest::Wasm(query) => match query {
                WasmQuery::Smart { contract_addr, msg } => {
                    // we want to make sure that contract queries only Lido Satellite
                    assert_eq!(contract_addr, "lido_satellite");
                    // we also want to make sure it only asks for its config
                    assert_eq!(msg, to_binary(&LidoSatelliteQueryConfig {}).unwrap());
                    SystemResult::Ok(ContractResult::from(to_binary(
                        &LidoSatelliteQueryConfigResponse {
                            bridged_denom: "bridged_denom".to_string(),
                            canonical_denom: "canonical_denom".to_string(),
                        },
                    )))
                }
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }
}

#[test]
fn success() {
    let mut deps = OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: CustomMockQuerier::default(),
        custom_query_type: PhantomData,
    };
    let response = instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("admin", &[]),
        InstantiateMsg {
            lido_satellite: "lido_satellite".to_string(),
            astroport_router: "astroport_router".to_string(),
        },
    )
    .unwrap();
    assert!(response.messages.is_empty());
    assert_eq!(
        response.attributes,
        vec![
            attr("action", "instantiate"),
            attr("lido_satellite", "lido_satellite"),
            attr("astroport_router", "astroport_router"),
            attr("bridged_denom", "bridged_denom"),
            attr("canonical_denom", "canonical_denom")
        ]
    );
    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        config,
        Config {
            lido_satellite: Addr::unchecked("lido_satellite"),
            astroport_router: Addr::unchecked("astroport_router"),
            bridged_denom: "bridged_denom".to_string(),
            canonical_denom: "canonical_denom".to_string(),
        }
    );
}
