use astroport::router::SwapOperation;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};
use neutron_sdk::bindings::msg::IbcFee;

#[cw_serde]
pub struct Config {
    pub lido_satellite: Addr,
    pub astroport_router: Addr,
}
pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct WrapAndSendContext {
    pub source_port: String,
    pub source_channel: String,
    pub receiver: String,
    pub astroport_swap_operations: Vec<SwapOperation>,
    pub refund_address: Addr,
    pub amount_to_wrap: Coin,
    pub amount_to_send: Coin,
    pub amount_to_swap_for_ibc_fee: Coin,
    pub ibc_fee_denom: String,
}
pub const WRAP_AND_SEND_CONTEXT: Item<WrapAndSendContext> = Item::new("wrap_and_send_context");

pub const IBC_FEE: Item<IbcFee> = Item::new("ibc_fee");

#[cw_serde]
pub struct IbcTransferInfo {
    pub refund_address: Addr,
    pub ibc_fee: IbcFee,
}

pub const IBC_TRANSFER_INFO: Map<
    (
        u64,  // sequence_id
        &str, // channel
    ),
    IbcTransferInfo,
> = Map::new("ibc_transfer_info");
