use crate::{ContractError, ContractResult};
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    /// This denom will be locked on contract's balance. Users are expected to send this
    /// denom with [`ExecuteMsg::Mint`] message in order to receive minted canonical funds.
    pub wsteth_denom: String,
    /// This subdenom will form a denom, minted by contract in exchange for wsteth sent by users.
    /// Users are expected to send this denom with [`ExecuteMsg::Burn`] message in order to
    /// receive original wsteth funds back.
    pub subdenom: String,
}

impl InstantiateMsg {
    pub fn validate(&self) -> ContractResult<()> {
        if self.wsteth_denom.is_empty() {
            return Err(ContractError::EmptyDenom {
                kind: "wsteth_denom".to_string(),
            });
        }
        if self.subdenom.is_empty() {
            return Err(ContractError::EmptyDenom {
                kind: "subdenom".to_string(),
            });
        }
        Ok(())
    }
}

#[cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
pub enum ExecuteMsg {
    /// This method expects user to send wsteth funds, which will be stored in the contract.
    /// User receives canonical wsteth funds, which are minted using tokenfactory module.
    Mint {
        /// By default, canonical wsteth is minted to sender, but it can be optionally minted
        /// to any address specified in this field.
        receiver: Option<String>,
    },
    /// This method expects user to send canonical wsteth funds, which will be burned.
    /// In exchange, user receives original wsteth funds back.
    Burn {
        /// By default, wsteth is returned back to sender, but it can be optionally returned
        /// to any address specified in this field.
        receiver: Option<String>,
    },
}

#[cw_serde]
pub struct ConfigResponse {
    pub wsteth_denom: String,
    pub subdenom: String,
}

#[cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

#[cw_serde]
pub struct MigrateMsg {}
