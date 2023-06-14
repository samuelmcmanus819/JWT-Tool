use cosmwasm_std::{
    StdError,
};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("Private key generation error")]
    KeygenError { failed_value: String },
    #[error("Failed to provision JWT")]
    ProvisionError {  },
}