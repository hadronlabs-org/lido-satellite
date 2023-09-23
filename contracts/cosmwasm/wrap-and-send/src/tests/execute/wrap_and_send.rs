use crate::{
    contract::execute,
    msg::ExecuteMsg,
    state::{WrapAndSendContext, WRAP_AND_SEND_CONTEXT},
    tests::helpers::instantiate_wrapper,
    ContractError,
};
use cosmwasm_std::{
    attr, coin, coins, from_slice, testing::mock_info, to_binary, Addr, BankMsg, ContractResult,
    CosmosMsg, Empty, Querier, QuerierResult, QueryRequest, SystemError, SystemResult, Uint128,
    WasmMsg, WasmQuery,
};
use lido_satellite::{
    error::ContractError as LidoSatelliteError,
    msg::{
        ConfigResponse as LidoSatelliteQueryConfigResponse,
        ExecuteMsg::Mint as LidoSatelliteExecuteMint, QueryMsg::Config as LidoSatelliteQueryConfig,
    },
};

#[derive(Default)]
struct CustomMockQuerier {}

impl Querier for CustomMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<Empty> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return QuerierResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
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

fn wrap_and_send_msg(amount_to_swap_for_ibc_fee: impl Into<Uint128>) -> ExecuteMsg {
    ExecuteMsg::WrapAndSend {
        source_port: "source_port".to_string(),
        source_channel: "source_channel".to_string(),
        receiver: "receiver".to_string(),
        amount_to_swap_for_ibc_fee: amount_to_swap_for_ibc_fee.into(),
        ibc_fee_denom: "ibc_fee_denom".to_string(),
        astroport_swap_operations: vec![],
        refund_address: "refund_address".to_string(),
    }
}

mod funds {
    use super::*;

    #[test]
    fn no_funds() {
        let (_result, mut deps, env) =
            instantiate_wrapper::<CustomMockQuerier>("lido_satellite", "astroport_router");
        let err = execute(
            deps.as_mut(),
            env,
            mock_info("stranger", &[]),
            wrap_and_send_msg(0u128),
        )
        .unwrap_err();
        assert_eq!(
            err,
            ContractError::LidoSatellite(LidoSatelliteError::NothingToMint {})
        );
    }

    #[test]
    fn wrong_denom() {
        let (_result, mut deps, env) =
            instantiate_wrapper::<CustomMockQuerier>("lido_satellite", "astroport_router");
        let err = execute(
            deps.as_mut(),
            env,
            mock_info("stranger", &[coin(200, "denom1")]),
            wrap_and_send_msg(0u128),
        )
        .unwrap_err();
        assert_eq!(
            err,
            ContractError::LidoSatellite(LidoSatelliteError::NothingToMint {})
        );
    }

    #[test]
    fn all_wrong_denoms() {
        let (_result, mut deps, env) =
            instantiate_wrapper::<CustomMockQuerier>("lido_satellite", "astroport_router");
        let err = execute(
            deps.as_mut(),
            env,
            mock_info("stranger", &[coin(200, "denom1"), coin(300, "denom2")]),
            wrap_and_send_msg(0u128),
        )
        .unwrap_err();
        assert_eq!(
            err,
            ContractError::LidoSatellite(LidoSatelliteError::ExtraFunds {})
        );
    }

    #[test]
    fn extra_denoms() {
        let (_result, mut deps, env) =
            instantiate_wrapper::<CustomMockQuerier>("lido_satellite", "astroport_router");
        let err = execute(
            deps.as_mut(),
            env,
            mock_info(
                "stranger",
                &[coin(200, "bridged_denom"), coin(300, "denom2")],
            ),
            wrap_and_send_msg(0u128),
        )
        .unwrap_err();
        assert_eq!(
            err,
            ContractError::LidoSatellite(LidoSatelliteError::ExtraFunds {})
        );
    }

    #[test]
    fn not_enough_for_ibc_fee() {
        let (_result, mut deps, env) =
            instantiate_wrapper::<CustomMockQuerier>("lido_satellite", "astroport_router");
        let response = execute(
            deps.as_mut(),
            env,
            mock_info("stranger", &[coin(200, "bridged_denom")]),
            wrap_and_send_msg(300u128),
        )
        .unwrap();
        assert_eq!(response.messages.len(), 1);
        assert_eq!(
            response.messages[0].msg,
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "refund_address".to_string(),
                amount: coins(200, "bridged_denom")
            })
        );
        assert_eq!(
            response.attributes,
            vec![
                attr("action", "cancel_wrap_and_send"),
                attr("reason", "not_enough_funds_to_swap"),
                attr("provided", "200bridged_denom"),
                attr("required", "300bridged_denom")
            ]
        );
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
        wrap_and_send_msg(100u128),
    )
    .unwrap();
    assert_eq!(response.messages.len(), 1);
    assert_eq!(
        response.messages[0].msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "lido_satellite".to_string(),
            msg: to_binary(&LidoSatelliteExecuteMint { receiver: None }).unwrap(),
            funds: coins(300, "bridged_denom"),
        })
    );
    assert_eq!(response.attributes, vec![attr("action", "wrap_and_send")]);
    let wrap_and_send_context = WRAP_AND_SEND_CONTEXT.load(deps.as_mut().storage).unwrap();
    assert_eq!(
        wrap_and_send_context,
        WrapAndSendContext {
            source_port: "source_port".to_string(),
            source_channel: "source_channel".to_string(),
            receiver: "receiver".to_string(),
            astroport_swap_operations: vec![],
            refund_address: Addr::unchecked("refund_address"),
            amount_to_wrap: coin(300, "bridged_denom"),
            amount_to_send: coin(200, "canonical_denom"),
            amount_to_swap_for_ibc_fee: coin(100, "canonical_denom"),
            ibc_fee_denom: "ibc_fee_denom".to_string(),
        }
    );
}