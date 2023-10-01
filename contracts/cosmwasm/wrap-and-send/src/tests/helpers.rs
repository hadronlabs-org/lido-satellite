use crate::{
    contract::instantiate,
    msg::{ExecuteMsg, InstantiateMsg},
    state::{Config, IbcTransferInfo, CONFIG, IBC_TRANSFER_INFO},
    ContractResult,
};
use cosmwasm_schema::serde::de::DeserializeOwned;
use cosmwasm_std::{
    coin, coins, from_slice,
    testing::{mock_env, mock_info, MockApi, MockStorage},
    Addr, Deps, DepsMut, Env, OwnedDeps, Querier, QuerierResult, QueryRequest, Response,
    SystemError, Uint128,
};
use neutron_sdk::{
    bindings::{
        msg::{IbcFee, NeutronMsg},
        query::NeutronQuery,
    },
    sudo::msg::RequestPacket,
};
use std::marker::PhantomData;

#[allow(clippy::type_complexity)]
pub fn instantiate_wrapper<Q: Querier + Default>(
    lido_satellite: impl Into<String>,
    astroport_router: impl Into<String>,
) -> (
    ContractResult<Response<NeutronMsg>>,
    OwnedDeps<MockStorage, MockApi, Q, NeutronQuery>,
    Env,
) {
    let mut deps = OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: Q::default(),
        custom_query_type: PhantomData,
    };
    let env = mock_env();
    (
        instantiate(
            deps.as_mut(),
            env.clone(),
            mock_info("admin", &[]),
            InstantiateMsg {
                lido_satellite: lido_satellite.into(),
                astroport_router: astroport_router.into(),
            },
        ),
        deps,
        env,
    )
}

pub fn assert_config(deps: Deps<NeutronQuery>, lido_satellite: &str, astroport_router: &str) {
    let config = CONFIG.load(deps.storage).unwrap();
    assert_eq!(
        config,
        Config {
            lido_satellite: Addr::unchecked(lido_satellite),
            astroport_router: Addr::unchecked(astroport_router),
        }
    )
}

pub fn craft_wrap_and_send_msg(amount_to_swap_for_ibc_fee: impl Into<Uint128>) -> ExecuteMsg {
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

pub fn craft_request_packet() -> RequestPacket {
    // almost all fields are set to None since we don't access them anyway
    RequestPacket {
        sequence: Some(4),
        source_port: None,
        source_channel: Some("chan".to_string()),
        destination_port: None,
        destination_channel: None,
        data: None,
        timeout_height: None,
        timeout_timestamp: None,
    }
}

pub fn prepare_ibc_transfer_info(deps: DepsMut<NeutronQuery>) {
    IBC_TRANSFER_INFO
        .save(
            deps.storage,
            (4, "chan"),
            &IbcTransferInfo {
                refund_address: Addr::unchecked("refund_address"),
                ibc_fee: IbcFee {
                    recv_fee: vec![],
                    ack_fee: coins(20, "ibc_fee_denom"),
                    timeout_fee: coins(30, "ibc_fee_denom"),
                },
                sent_amount: coin(500, "canonical_denom"),
            },
        )
        .unwrap();
}

pub fn bin_request_to_query_request<T: DeserializeOwned>(
    bin_request: &[u8],
) -> Result<QueryRequest<T>, QuerierResult> {
    from_slice::<QueryRequest<T>>(bin_request).map_err(move |err| {
        QuerierResult::Err(SystemError::InvalidRequest {
            error: format!("Parsing query request: {}", err),
            request: bin_request.into(),
        })
    })
}
