use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("denom field {kind} should not be empty")]
    EmptyDenom { kind: String },

    #[error("denom {denom} is not a correct IBC denom: {reason}")]
    InvalidIbcDenom { denom: String, reason: String },

    #[error("nothing to burn: canonical funds should be provided")]
    NothingToBurn {},

    #[error("nothing to mint: bridged funds should be provided")]
    NothingToMint {},

    #[error("extra funds have been supplied")]
    ExtraFunds {},
}

pub type ContractResult<T> = Result<T, ContractError>;
