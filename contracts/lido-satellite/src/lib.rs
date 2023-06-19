pub mod contract;
pub mod error;
pub mod msg;
pub mod state;

pub use crate::error::{ContractError, ContractResult};

#[cfg(feature = "interface")]
pub use crate::{
    contract::LidoSatellite,
    msg::{ExecuteMsgFns as LidoSatelliteExecuteMsgFns, QueryMsgFns as LidoSatelliteQueryMsgFns},
};

mod execute;
mod query;

#[cfg(test)]
mod tests;
