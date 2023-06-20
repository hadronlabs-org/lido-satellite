use crate::{ContractError, ContractResult};
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    /// This denom will be locked on contract's balance. Users are expected to send this
    /// denom with [`ExecuteMsg::Mint`] message in order to receive minted canonical funds.
    pub bridged_denom: String,
    /// This subdenom will form a canonical denom, minted by contract in exchange for bridged funds
    /// sent by users. Users are expected to send this denom with [`ExecuteMsg::Burn`] message
    /// in order to receive original bridged funds back.
    pub canonical_subdenom: String,
}

impl InstantiateMsg {
    pub fn validate(&self) -> ContractResult<()> {
        if self.bridged_denom.is_empty() {
            return Err(ContractError::EmptyDenom {
                kind: "bridged_denom".to_string(),
            });
        }
        if self.canonical_subdenom.is_empty() {
            return Err(ContractError::EmptyDenom {
                kind: "canonical_subdenom".to_string(),
            });
        }
        Ok(())
    }
}

#[cw_serde]
pub enum ExecuteMsg {
    /// This method expects users to send bridged funds, which will be locked in contract.
    /// In exchange, users receive canonical funds, which are minted by tokenfactory module.
    Mint {
        /// By default, canonical funds are minted to sender, but they can optionally be minted
        /// to any address specified in this field.
        receiver: Option<String>,
    },
    /// This method expects users to send canonical funds, which will be burned.
    /// In exchange, users receive original bridged funds back.
    Burn {
        /// By default, bridged funds are returned back to sender, but they can optionally be
        /// returned to any address specified in this field.
        receiver: Option<String>,
    },
}

#[cw_serde]
pub struct ConfigResponse {
    pub bridged_denom: String,
    pub canonical_subdenom: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

#[cw_serde]
pub struct MigrateMsg {}
