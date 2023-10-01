use crate::{
    contract::execute,
    tests::helpers::{bin_request_to_query_request, craft_wrap_and_send_msg, instantiate_wrapper},
};
use cosmwasm_std::{
    coin, testing::mock_info, to_binary, ContractResult, Querier, QuerierResult, QueryRequest,
    SystemResult, WasmQuery,
};
use lido_satellite::msg::{
    ConfigResponse as LidoSatelliteQueryConfigResponse,
    QueryMsg::Config as LidoSatelliteQueryConfig,
};
use neutron_sdk::{
    bindings::{msg::IbcFee, query::NeutronQuery},
    query::min_ibc_fee::MinIbcFeeResponse,
};

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
            QueryRequest::Custom(query) => match query {
                NeutronQuery::MinIbcFee {} => {
                    SystemResult::Ok(ContractResult::from(to_binary(&MinIbcFeeResponse {
                        min_fee: IbcFee {
                            recv_fee: vec![],
                            ack_fee: vec![coin(20, "ibc_fee_denom"), coin(45, "untrn")],
                            timeout_fee: vec![coin(30, "ibc_fee_denom"), coin(55, "untrn")],
                        },
                    })))
                }
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }
}

mod funds {
    use super::*;

    #[test]
    fn no_funds() {
        let (_result, mut deps, env) =
            instantiate_wrapper::<CustomMockQuerier>("lido_satellite", "astroport_router");
        execute(
            deps.as_mut(),
            env,
            mock_info("stranger", &[]),
            craft_wrap_and_send_msg(0u128),
        )
        .unwrap_err();
    }

    #[test]
    fn wrong_denom() {
        let (_result, mut deps, env) =
            instantiate_wrapper::<CustomMockQuerier>("lido_satellite", "astroport_router");
        execute(
            deps.as_mut(),
            env,
            mock_info("stranger", &[coin(200, "denom1")]),
            craft_wrap_and_send_msg(0u128),
        )
        .unwrap_err();
    }

    #[test]
    fn all_wrong_denoms() {
        let (_result, mut deps, env) =
            instantiate_wrapper::<CustomMockQuerier>("lido_satellite", "astroport_router");
        execute(
            deps.as_mut(),
            env,
            mock_info("stranger", &[coin(200, "denom1"), coin(300, "denom2")]),
            craft_wrap_and_send_msg(0u128),
        )
        .unwrap_err();
    }

    #[test]
    fn extra_denoms() {
        let (_result, mut deps, env) =
            instantiate_wrapper::<CustomMockQuerier>("lido_satellite", "astroport_router");
        execute(
            deps.as_mut(),
            env,
            mock_info(
                "stranger",
                &[coin(200, "bridged_denom"), coin(300, "denom2")],
            ),
            craft_wrap_and_send_msg(0u128),
        )
        .unwrap_err();
    }

    #[test]
    fn not_enough_for_ibc_fee() {
        let (_result, mut deps, env) =
            instantiate_wrapper::<CustomMockQuerier>("lido_satellite", "astroport_router");
        execute(
            deps.as_mut(),
            env,
            mock_info("stranger", &[coin(200, "bridged_denom")]),
            craft_wrap_and_send_msg(300u128),
        )
        .unwrap_err();
    }
}

#[test]
fn success() {
    let (_result, mut deps, env) =
        instantiate_wrapper::<CustomMockQuerier>("lido_satellite", "astroport_router");
    let response = execute(
        deps.as_mut(),
        env,
        mock_info("stranger", &[coin(300, "bridged_denom")]),
        craft_wrap_and_send_msg(100u128),
    )
    .unwrap();
    dbg!(response);
    // TODO: there should be lots of messages, check them all
    // TODO: check attributes as well
}
