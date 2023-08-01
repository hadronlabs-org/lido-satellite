use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub bridged_denom: String,
    pub canonical_denom: String,
}

pub const CONFIG: Item<Config> = Item::new("config");
