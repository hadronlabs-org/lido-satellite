use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub wrap_and_send: Addr,
    pub lido_satellite: Addr,
    pub bridged_denom: String,
    pub canonical_denom: String,
}
pub const CONFIG: Item<Config> = Item::new("config");

pub const REFUND_ADDRESS: Item<Addr> = Item::new("refund_address");
pub const FUNDS: Item<Coin> = Item::new("funds");
