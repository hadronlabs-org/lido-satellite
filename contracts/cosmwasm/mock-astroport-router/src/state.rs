use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub offer_denom: String,
    pub ask_denom: String,
}
pub const CONFIG: Item<Config> = Item::new("config");
