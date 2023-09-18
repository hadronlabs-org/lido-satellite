use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    /// Address of Lido Satellite contract, used to mint canonical wstETH
    pub lido_satellite: String,
    // TODO: make owner be able to set this denom
    // TODO: make owner be able to withdraw native tokens from the contract
    /// Denom to be used to pay for IBC fees
    pub ibc_fee_denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// This method expects user to send bridged funds, which will be sent to Lido Satellite
    /// and locked in there. Lido Satellite will mint canonical wstETH in return, which will
    /// be sent further to another destination chain
    WrapAndSend {
        /// Source port to send funds from
        source_port: String,
        /// Source channel to send funds from
        source_channel: String,
        /// Address of the receiver on a remote chain
        receiver: String,
    },
}

#[cw_serde]
pub struct ConfigResponse {
    pub lido_satellite: String,
    pub ibc_fee_denom: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

#[cw_serde]
pub struct MigrateMsg {}
