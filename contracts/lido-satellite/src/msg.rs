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
        Self::validate_ibc_denom(&self.bridged_denom)?;

        if self.canonical_subdenom.is_empty() {
            return Err(ContractError::EmptyDenom {
                kind: "canonical_subdenom".to_string(),
            });
        }
        Ok(())
    }

    fn validate_ibc_denom(denom: &str) -> ContractResult<()> {
        let invalid_denom = |reason: &str| {
            Err(ContractError::InvalidIbcDenom {
                denom: String::from(denom),
                reason: reason.to_string(),
            })
        };

        // Example IBC denom: ibc/584A4A23736884E0C198FD1EE932455A9357A492A7B94324E4A02B5628687831
        // Length of this string is `len("ibc/") + 64 /* hex encoded 32 bytes hash */ == 68`

        // Step 1: Validate length
        if denom.len() != 68 {
            return invalid_denom("expected length of 68 chars");
        }

        // Step 2: Validate prefix
        if !denom.starts_with("ibc/") {
            return invalid_denom("expected prefix 'ibc/'");
        }

        // Step 3: Validate hash
        if !denom
            .chars()
            .skip(4)
            // c.is_ascii_hexdigit() could have been used here, but it allows lowercase characters
            .all(|c| matches!(c, '0'..='9' | 'A'..='F'))
        {
            return invalid_denom("invalid denom hash");
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
    pub canonical_denom: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

#[cw_serde]
pub struct MigrateMsg {}
