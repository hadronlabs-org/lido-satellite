use cosmwasm_schema::{cw_serde, QueryResponses};

pub use wrap_and_send::msg::ExecuteMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub wrap_and_send: String,
}

// TODO: implement query()
// TODO: implement test for this query
#[cw_serde]
pub struct ConfigResponse {
    pub wrap_and_send: String,
    pub lido_satellite: String,
    pub bridged_denom: String,
    pub canonical_denom: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}
