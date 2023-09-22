use crate::{contract::execute, msg::ExecuteMsg, tests::helpers::instantiate_wrapper};
use cosmwasm_std::{
    from_slice, testing::mock_info, Binary, Empty, Querier, QuerierResult, QueryRequest,
    SystemError, Uint128, WasmQuery,
};

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
                    // we want to make sure that contract only calls lido satellite for query
                    assert_eq!(contract_addr, "lido_satellite");
                    // we also want to make sure it only asks for a config
                    // TODO: use Lido Satellite query message to_binary
                    assert_eq!(msg, Binary::from(r#"{"config":{}}"#.as_bytes()));
                    todo!()
                }
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }
}

impl Default for CustomMockQuerier {
    fn default() -> Self {
        Self {}
    }
}

#[test]
fn no_funds() {
    // TODO: mock query lido satellite config
    let (_result, mut deps, env) =
        instantiate_wrapper::<CustomMockQuerier>("lido_satellite", "aatroport_router");
    let err = execute(
        deps.as_mut(),
        env,
        mock_info("stranger", &[]),
        ExecuteMsg::WrapAndSend {
            source_port: "source_port".to_string(),
            source_channel: "source_channel".to_string(),
            receiver: "receiver".to_string(),
            amount_to_swap_for_ibc_fee: Uint128::zero(),
            ibc_fee_denom: "ibc_fee_denom".to_string(),
            astroport_swap_operations: vec![],
            refund_address: "refund_address".to_string(),
        },
    )
    .unwrap_err();
    dbg!(err);
}
