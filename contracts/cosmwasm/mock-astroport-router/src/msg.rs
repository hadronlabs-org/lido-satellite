use cosmwasm_schema::{cw_serde, QueryResponses};

pub use astroport::router::ExecuteMsg;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
