use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub wrap_and_send: Addr,
}
pub const CONFIG: Item<Config> = Item::new("config");

pub const REFUND_ADDRESS: Item<Addr> = Item::new("refund_address");
pub const FUNDS: Item<Vec<Coin>> = Item::new("funds");
