use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};
use neutron_sdk::bindings::msg::IbcFee;

#[cw_serde]
pub struct Config {
    pub lido_satellite: Addr,
    pub astroport_router: Addr,
    pub bridged_denom: String,
    pub canonical_denom: String,
}
pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct IbcTransferInfo {
    pub refund_address: Addr,
    pub ibc_fee: IbcFee,
    pub sent_amount: Coin,
}

// reentrance protection
pub const EXECUTION_FLAG: Item<bool> = Item::new("execution_flag");

// TODO: combine this into a structure? structure name will also document it
// temporary state used to refund canonical denoms in case if contract fails
pub const REFUND_ADDRESS: Item<Addr> = Item::new("refund_address");
pub const FUNDS: Item<Coin> = Item::new("funds");

// temporary state used to restore context after a call to IBC transfer module
pub const IBC_TRANSFER_CONTEXT: Item<IbcTransferInfo> = Item::new("ibc_transfer_context");

// persistent state used to refund failed IBC transfers and IBC fees
pub const IBC_TRANSFER_INFO: Map<
    (
        u64,  // sequence_id
        &str, // channel
    ),
    IbcTransferInfo,
> = Map::new("ibc_transfer_info");
