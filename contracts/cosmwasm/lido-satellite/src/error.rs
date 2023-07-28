use cosmwasm_std::{OverflowError, StdError};
use neutron_sdk::NeutronError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    NeutronError(#[from] NeutronError),

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

    #[error("unknown reply id: {id}")]
    UnknownReplyId { id: u64 },
}

pub type ContractResult<T> = Result<T, ContractError>;
