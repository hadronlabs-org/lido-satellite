use crate::{
    state::{IBC_TRANSFER_CONTEXT, IBC_TRANSFER_INFO},
    ContractResult,
};
use cosmwasm_std::{attr, from_binary, DepsMut, Env, Response, SubMsgResult};
use neutron_sdk::bindings::{
    msg::{MsgIbcTransferResponse, NeutronMsg},
    query::NeutronQuery,
};

pub(crate) fn reply_ibc_transfer(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    result: SubMsgResult,
) -> ContractResult<Response<NeutronMsg>> {
    // Step 4.
    // Handle immediate reply from IBC transfer module
    // On failure: refund canonical funds and IBC fees back to user
    // On success: store sequence_id and channel to handle IBC callback later

    let context = IBC_TRANSFER_CONTEXT.load(deps.storage)?;
    IBC_TRANSFER_CONTEXT.remove(deps.storage);

    match result {
        // we request reply on success, so we don't have to implement failure handling
        SubMsgResult::Err(_e) => unimplemented!(),
        SubMsgResult::Ok(response) => {
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
        }
    }
}
