use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;

#[cw_serde]
pub struct InstantiateMsg {
    /// Address of Lido Satellite contract, used to mint canonical wstETH
    pub lido_satellite: String,
    /// Denom to be used to pay for IBC fees
    pub ibc_fee_denom: String,
    /// Owner is able to set denom used to pay for IBC fees.
    /// Owner is also able to withdraw funds from contract's account.
    pub owner: Option<String>,
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
    /// Can only be called by an owner
    SetOwner { new_owner: Option<String> },
    /// Can only be called by owner
    SetIbcFeeDenom { new_ibc_fee_denom: String },
    /// Can only be called by owner
    WithdrawFunds {
        funds: Coin,
        /// Recipient of withdrawn funds, defaults to message sender if not set
        receiver: Option<String>,
    },
}

#[cw_serde]
pub struct ConfigResponse {
    pub lido_satellite: String,
    pub ibc_fee_denom: String,
    pub owner: Option<String>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

#[cw_serde]
pub struct MigrateMsg {}
