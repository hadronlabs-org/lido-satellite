#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] cosmwasm_std::StdError),

    #[error("{0}")]
    OverflowError(#[from] cosmwasm_std::OverflowError),

    #[error("{0}")]
    NeutronError(#[from] neutron_sdk::NeutronError),

    #[error("{0}")]
    LidoSatellite(#[from] lido_satellite::error::ContractError),

    #[error("this method is only callable by contract itself")]
    InternalMethod {},

    #[error("Astroport Router provided less funds than requested")]
    SwappedForLessThanRequested {},

    #[error("unknown reply id: {id}")]
    UnknownReplyId { id: u64 },

    #[error("couldn't calculate minimum IBC fee")]
    MinIbcFee {},

    #[error("already in execution: reentrance is not allowed")]
    AlreadyInExecution {},

    #[error("amount_to_swap_for_ibc_fee is zero")]
    ZeroForSwap {},

    #[error("supplied funds are not enough to perform swap")]
    NotEnoughForSwap {},
}

pub type ContractResult<T> = Result<T, ContractError>;
