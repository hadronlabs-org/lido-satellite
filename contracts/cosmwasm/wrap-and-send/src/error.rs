use cosmwasm_std::StdError;
use lido_satellite::error::ContractError as LidoSatelliteError;
use neutron_sdk::NeutronError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    NeutronError(#[from] NeutronError),

    #[error("{0}")]
    LidoSatellite(#[from] LidoSatelliteError),
}

pub type ContractResult<T> = Result<T, ContractError>;
