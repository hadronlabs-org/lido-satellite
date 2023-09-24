use crate::{
    contract::instantiate,
    msg::{ExecuteMsg, InstantiateMsg},
    state::{Config, WrapAndSendContext, CONFIG},
    ContractResult,
};
use cosmwasm_std::{
    coin,
    testing::{mock_env, mock_info, MockApi, MockStorage},
    Addr, Deps, Env, OwnedDeps, Querier, Response, Uint128,
};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};
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

pub fn craft_wrap_and_send_context() -> WrapAndSendContext {
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
}
