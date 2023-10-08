use crate::{
    msg::ExecuteMsg,
    state::{Config, IbcTransferInfo, CONFIG, IBC_TRANSFER_INFO},
};
use cosmwasm_schema::serde::de::DeserializeOwned;
use cosmwasm_std::{
    coin, coins, from_slice,
    testing::{mock_env, MockApi, MockStorage},
    Addr, Deps, DepsMut, Env, OwnedDeps, Querier, QuerierResult, QueryRequest, SystemError,
    Uint128,
};
use neutron_sdk::{
    bindings::{msg::IbcFee, query::NeutronQuery},
    sudo::msg::RequestPacket,
};
use std::marker::PhantomData;

#[allow(clippy::type_complexity)]
pub fn mock_instantiate<Q: Querier + Default>(
) -> (OwnedDeps<MockStorage, MockApi, Q, NeutronQuery>, Env) {
    let mut deps = OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: Q::default(),
        custom_query_type: PhantomData,
    };
    let env = mock_env();
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                lido_satellite: Addr::unchecked("lido_satellite".to_string()),
                astroport_router: Addr::unchecked("astroport_router".to_string()),
                bridged_denom: "bridged_denom".to_string(),
                canonical_denom: "canonical_denom".into(),
            },
        )
        .unwrap();
    (deps, env)
}

pub fn assert_config(
    deps: Deps<NeutronQuery>,
    lido_satellite: &str,
    astroport_router: &str,
    bridged_denom: &str,
    canonical_denom: &str,
) {
    let config = CONFIG.load(deps.storage).unwrap();
    assert_eq!(
        config,
        Config {
            lido_satellite: Addr::unchecked(lido_satellite),
            astroport_router: Addr::unchecked(astroport_router),
            bridged_denom: bridged_denom.to_string(),
            canonical_denom: canonical_denom.to_string(),
        }
    )
}

pub fn craft_wrap_and_send_msg() -> ExecuteMsg {
    ExecuteMsg::WrapAndSend {
        source_port: "source_port".to_string(),
        source_channel: "source_channel".to_string(),
        receiver: "receiver".to_string(),
        amount_to_swap_for_ibc_fee: Uint128::new(0),
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
