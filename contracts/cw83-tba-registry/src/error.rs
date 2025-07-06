use cosmwasm_std::StdError;
use cw_utils::{ParseReplyError, PaymentError};
use saa_wasm::saa_types::AuthError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("{0}")]
    Auth(#[from] AuthError),

    #[error("Insufficient fee: expected {0}, got {1}")]
    InsufficientFee(u128, u128),

    #[error("None of the sent tokens are accepted by the contract")]
    NoFeeTokens {},

    #[error("Code ids are invalid or not provided")]
    InvalidCodeIds {},

    #[error("Must provide at least one correct creating fee")]
    InvalidCreationFees {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Not Supported Chain ID")]
    InvalidChainId {},

    #[error("Code ID not allowed")]
    InvalidCodeId {},

    #[error("Account for the given token already exists. Use `reset_account` to overwrite it and `migrate_account` to update it to a newer version")]
    AccountExists {},

    #[error("Generic error: {0}")]
    Generic(String),

    #[error("Semver parsing error: {0}")]
    SemVer(String),
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}

impl ContractError {
    pub fn generic(msg: String) -> Self {
        Self::Generic(msg)
    }
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}

impl From<ParseReplyError> for ContractError {
    fn from(err: ParseReplyError) -> Self {
        Self::Std(StdError::generic_err(err.to_string()))
    }
}
