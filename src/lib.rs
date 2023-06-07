mod error;
mod execute;
mod instantiate;
mod query;

pub mod msg;
pub mod state;
pub use crate::error::{ContractError, ContractResult};

pub mod contract {
    pub use super::{execute::execute, instantiate::instantiate, query::query};
}

#[cfg(test)]
mod tests;
