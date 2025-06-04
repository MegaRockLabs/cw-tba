use saa_wasm::saa_types::{AuthError, SessionError, errors::StorageError};
use cw_ownable::OwnershipError;
use cosmwasm_std::StdError;
use thiserror::Error;


#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Ownership(#[from] OwnershipError),

    #[error("{0}")]
    Auth(#[from] AuthError),

    #[error("{0}")]
    Session(#[from] SessionError),

    #[error("{0}")]
    Storage(#[from] StorageError),

    #[error("This method can only be called from the registry contract")]
    NotRegistry {},

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Not Supported")]
    NotSupported {},

    #[error("Invalid signed action: {0}")]
    BadSignedAction(String),

    #[error("Account is frozen until ownership or credentials updates")]
    Frozen {},

    #[error("Deleted")]
    Deleted {},

    #[error("Provided nonce has already been used")]
    NonceExists {},

    #[error("At least one of the provided credentials must be deriving into owner of the token")]
    NoOwnerCred {},

    #[error("Can't call this method directly. Must be signed.")]
    NoDirectCall {},

    #[error("{0}")]
    Generic(String),

    #[error("Semver parsing error: {0}")]
    SemVer(String),
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
