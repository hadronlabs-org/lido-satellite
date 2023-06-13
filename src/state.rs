use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub bridged_denom: String,
    pub canonical_subdenom: String,
}

impl Config {
    pub fn get_full_tokenfactory_denom(&self, contract_address: impl AsRef<str>) -> String {
        format!(
            "factory/{}/{}",
            contract_address.as_ref(),
            self.canonical_subdenom
        )
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
