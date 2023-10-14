use crate::{
    state::{EXECUTION_FLAG, IBC_TRANSFER_CONTEXT, IBC_TRANSFER_INFO, REFUND_INFO},
    ContractResult,
};
use cosmwasm_std::{attr, from_binary, BankMsg, CosmosMsg, DepsMut, Env, Response, SubMsgResult};
use neutron_sdk::bindings::{
    msg::{MsgIbcTransferResponse, NeutronMsg},
    query::NeutronQuery,
};

pub fn reply_wrap_callback(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    result: SubMsgResult,
) -> ContractResult<Response<NeutronMsg>> {
    if let SubMsgResult::Err(_) = result {
        EXECUTION_FLAG.remove(deps.storage);

        let refund_info = REFUND_INFO.load(deps.storage)?;

        let send_msg: CosmosMsg<NeutronMsg> = BankMsg::Send {
            to_address: refund_info.refund_address.into_string(),
            amount: vec![refund_info.funds],
        }
        .into();

        // TODO: attributes
        Ok(Response::new().add_message(send_msg))
    } else {
        unreachable!("because we use `SubMsg::reply_on_error`")
    }
}

pub(crate) fn reply_ibc_transfer(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    result: SubMsgResult,
) -> ContractResult<Response<NeutronMsg>> {
    if let SubMsgResult::Ok(response) = result {
        EXECUTION_FLAG.remove(deps.storage);

        let context = IBC_TRANSFER_CONTEXT.load(deps.storage)?;
        IBC_TRANSFER_CONTEXT.remove(deps.storage);

        // IBC transfer module always sets reply data on success:
        // https://github.com/neutron-org/neutron/blob/v1.0.4/x/transfer/keeper/keeper.go#L27-L62
        // hence I can easily use `Option::unwrap()` here
        let data = response.data.unwrap();
        let response: MsgIbcTransferResponse = from_binary(&data)?;
        IBC_TRANSFER_INFO.save(
            deps.storage,
            (response.sequence_id, &response.channel),
            &context,
        )?;
        Ok(Response::new().add_attributes([
            attr("subaction", "ibc_transfer"),
            attr("sequence_id", response.sequence_id.to_string()),
            attr("channel", response.channel),
        ]))
    } else {
        unreachable!("because we use `SubMsg::reply_on_success`")
    }
}
