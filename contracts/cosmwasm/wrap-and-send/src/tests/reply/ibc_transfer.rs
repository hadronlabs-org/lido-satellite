use crate::{
    contract::{reply, IBC_TRANSFER_REPLY_ID},
    state::{IbcTransferInfo, EXECUTION_FLAG, IBC_TRANSFER_CONTEXT, IBC_TRANSFER_INFO},
    tests::helpers::mock_instantiate,
};
use cosmwasm_std::{
    attr, coin, coins, testing::MockQuerier, to_binary, Addr, Reply, SubMsgResponse, SubMsgResult,
};
use neutron_sdk::bindings::msg::{IbcFee, MsgIbcTransferResponse};

#[test]
fn success() {
    let (mut deps, env) = mock_instantiate::<MockQuerier>();
    EXECUTION_FLAG.save(deps.as_mut().storage, &true).unwrap();
    IBC_TRANSFER_CONTEXT
        .save(deps.as_mut().storage, &craft_ibc_transfer_info())
        .unwrap();
    let response = reply(
        deps.as_mut(),
        env,
        Reply {
            id: IBC_TRANSFER_REPLY_ID,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                data: Some(
                    to_binary(&MsgIbcTransferResponse {
                        sequence_id: 6,
                        channel: "channel-8".to_string(),
                    })
                    .unwrap(),
                ),
            }),
        },
    )
    .unwrap();

    let execution_flag = EXECUTION_FLAG.may_load(deps.as_ref().storage).unwrap();
    assert_eq!(execution_flag, None);
    let ibc_transfer_context = IBC_TRANSFER_CONTEXT
        .may_load(deps.as_ref().storage)
        .unwrap();
    assert_eq!(ibc_transfer_context, None);
    let ibc_transfer_info = IBC_TRANSFER_INFO
        .load(deps.as_ref().storage, (6, "channel-8"))
        .unwrap();
    assert_eq!(ibc_transfer_info, craft_ibc_transfer_info());

    assert!(response.messages.is_empty());

    assert_eq!(response.attributes, vec![attr("ibc_sequence_id", "6")]);
}

fn craft_ibc_transfer_info() -> IbcTransferInfo {
    IbcTransferInfo {
        refund_address: Addr::unchecked("refund_address"),
        ibc_fee: IbcFee {
            recv_fee: vec![],
            ack_fee: coins(20, "ibc_fee_denom"),
            timeout_fee: coins(40, "ibc_fee_denom"),
        },
        sent_amount: coin(220, "canonical_denom"),
    }
}
