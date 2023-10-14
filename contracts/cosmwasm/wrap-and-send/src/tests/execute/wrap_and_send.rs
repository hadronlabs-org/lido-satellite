use crate::{
    contract::{execute, WRAP_CALLBACK_REPLY_ID},
    msg::ExecuteMsg,
    state::{RefundInfo, EXECUTION_FLAG, REFUND_INFO},
    tests::helpers::mock_instantiate,
    ContractError,
};
use astroport::router::SwapOperation;
use cosmwasm_std::{
    attr, coin, coins,
    testing::{mock_info, MockQuerier, MOCK_CONTRACT_ADDR},
    to_binary, Addr, CosmosMsg, ReplyOn, Uint128, WasmMsg,
};
use lido_satellite::{
    msg::ExecuteMsg::Mint as LidoSatelliteExecuteMint, ContractError as LidoSatelliteError,
};

mod invalid_funds {
    use super::*;

    #[test]
    fn no_funds() {
        let (mut deps, env) = mock_instantiate::<MockQuerier>();
        let err = execute(
            deps.as_mut(),
            env,
            mock_info("stranger", &[]),
            craft_wrap_and_send_msg(0),
        )
        .unwrap_err();
        assert_eq!(
            err,
            ContractError::LidoSatellite(LidoSatelliteError::NothingToMint {})
        );
    }

    #[test]
    fn wrong_denom() {
        let (mut deps, env) = mock_instantiate::<MockQuerier>();
        let err = execute(
            deps.as_mut(),
            env,
            mock_info("stranger", &[coin(200, "denom1")]),
            craft_wrap_and_send_msg(0),
        )
        .unwrap_err();
        assert_eq!(
            err,
            ContractError::LidoSatellite(LidoSatelliteError::NothingToMint {})
        );
    }

    #[test]
    fn all_wrong_denoms() {
        let (mut deps, env) = mock_instantiate::<MockQuerier>();
        let err = execute(
            deps.as_mut(),
            env,
            mock_info("stranger", &[coin(200, "denom1"), coin(300, "denom2")]),
            craft_wrap_and_send_msg(0),
        )
        .unwrap_err();
        assert_eq!(
            err,
            ContractError::LidoSatellite(LidoSatelliteError::ExtraFunds {})
        );
    }

    #[test]
    fn extra_denoms() {
        let (mut deps, env) = mock_instantiate::<MockQuerier>();
        let err = execute(
            deps.as_mut(),
            env,
            mock_info(
                "stranger",
                &[coin(200, "bridged_denom"), coin(300, "denom2")],
            ),
            craft_wrap_and_send_msg(0),
        )
        .unwrap_err();
        assert_eq!(
            err,
            ContractError::LidoSatellite(LidoSatelliteError::ExtraFunds {})
        );
    }
}

#[test]
fn reentrance_protection() {
    let (mut deps, env) = mock_instantiate::<MockQuerier>();
    EXECUTION_FLAG.save(deps.as_mut().storage, &true).unwrap();
    let err = execute(
        deps.as_mut(),
        env,
        mock_info("stranger", &[coin(300, "bridged_denom")]),
        craft_wrap_and_send_msg(0),
    )
    .unwrap_err();
    assert_eq!(err, ContractError::AlreadyInExecution {});
}

#[test]
fn success() {
    let (mut deps, env) = mock_instantiate::<MockQuerier>();
    let response = execute(
        deps.as_mut(),
        env,
        mock_info("stranger", &[coin(300, "bridged_denom")]),
        craft_wrap_and_send_msg(100),
    )
    .unwrap();

    let execution_flag = EXECUTION_FLAG.load(deps.as_ref().storage).unwrap();
    assert!(execution_flag);
    let refund_info = REFUND_INFO.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        refund_info,
        RefundInfo {
            refund_address: Addr::unchecked("refund_address"),
            funds: coin(300, "canonical_denom"),
        }
    );

    assert_eq!(response.messages.len(), 2);
    assert_eq!(
        response.messages[0].msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "lido_satellite".to_string(),
            msg: to_binary(&LidoSatelliteExecuteMint { receiver: None }).unwrap(),
            funds: coins(300, "bridged_denom"),
        })
    );
    assert_eq!(response.messages[1].id, WRAP_CALLBACK_REPLY_ID);
    assert_eq!(response.messages[1].reply_on, ReplyOn::Error);
    assert_eq!(
        response.messages[1].msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_CONTRACT_ADDR.to_string(),
            msg: to_binary(&ExecuteMsg::WrapCallback {
                source_port: "source_port".to_string(),
                source_channel: "source_channel".to_string(),
                receiver: "receiver".to_string(),
                amount_to_swap_for_ibc_fee: Uint128::new(100),
                ibc_fee_denom: "ibc_fee_denom".to_string(),
                astroport_swap_operations: vec![SwapOperation::NativeSwap {
                    offer_denom: "canonical_denom".to_string(),
                    ask_denom: "ibc_fee_denom".to_string(),
                }],
                received_amount: Uint128::new(300),
                refund_address: Addr::unchecked("refund_address"),
            })
            .unwrap(),
            funds: vec![],
        })
    );

    assert_eq!(
        response.attributes,
        vec![
            attr("action", "wrap_and_send"),
            attr("received_amount", "300bridged_denom"),
            attr("refund_address", "refund_address"),
        ]
    );
}

fn craft_wrap_and_send_msg(amount_to_swap_for_ibc_fee: u128) -> ExecuteMsg {
    ExecuteMsg::WrapAndSend {
        source_port: "source_port".to_string(),
        source_channel: "source_channel".to_string(),
        receiver: "receiver".to_string(),
        amount_to_swap_for_ibc_fee: amount_to_swap_for_ibc_fee.into(),
        ibc_fee_denom: "ibc_fee_denom".to_string(),
        astroport_swap_operations: vec![SwapOperation::NativeSwap {
            offer_denom: "canonical_denom".to_string(),
            ask_denom: "ibc_fee_denom".to_string(),
        }],
        refund_address: "refund_address".to_string(),
    }
}
