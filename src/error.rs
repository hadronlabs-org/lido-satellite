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

    #[error("nothing to burn: canonical wsteth funds should be provided")]
    NothingToBurn {},

    #[error("nothing to mint: wsteth funds should be provided")]
    NothingToMint {},
}

pub type ContractResult<T> = Result<T, ContractError>;
