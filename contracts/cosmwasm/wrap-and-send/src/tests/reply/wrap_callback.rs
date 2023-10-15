use crate::{
    contract::{reply, WRAP_CALLBACK_REPLY_ID},
    state::{RefundInfo, EXECUTION_FLAG, REFUND_INFO},
    tests::helpers::mock_instantiate,
};
use cosmwasm_std::{
    attr, coin, coins, testing::MockQuerier, Addr, BankMsg, CosmosMsg, Reply, SubMsgResult,
};

#[test]
fn failure() {
    let (mut deps, env) = mock_instantiate::<MockQuerier>();
    EXECUTION_FLAG.save(deps.as_mut().storage, &true).unwrap();
    REFUND_INFO
        .save(
            deps.as_mut().storage,
            &RefundInfo {
                refund_address: Addr::unchecked("refund_address"),
                funds: coin(50, "canonical_denom"),
            },
        )
        .unwrap();
    let response = reply(
        deps.as_mut(),
        env,
        Reply {
            id: WRAP_CALLBACK_REPLY_ID,
            result: SubMsgResult::Err("codespace: wasm, code: 21".to_string()),
        },
    )
    .unwrap();

    let execution_flag = EXECUTION_FLAG.may_load(deps.as_ref().storage).unwrap();
    assert_eq!(execution_flag, None);
    let refund_info = REFUND_INFO.may_load(deps.as_ref().storage).unwrap();
    assert_eq!(refund_info, None);

    assert_eq!(response.messages.len(), 1);
    assert_eq!(
        response.messages[0].msg,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "refund_address".to_string(),
            amount: coins(50, "canonical_denom"),
        }),
    );

    assert_eq!(
        response.attributes,
        vec![
            attr("error", "codespace: wasm, code: 21"),
            attr("action", "refund"),
            attr("refund_amount", "50canonical_denom")
        ]
    );
}
