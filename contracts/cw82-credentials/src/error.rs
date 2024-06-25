use cosmwasm_std::StdError;
use cw_ownable::OwnershipError;
use cw_utils::ParseReplyError;
use saa::AuthError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Ownership(#[from] OwnershipError),

    #[error("{0}")]
    Parse(#[from] ParseReplyError),

    #[error("{0}")]
    Auth(#[from] AuthError),

    #[error("Invalid Payload: {0}")]
    Payload(String),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Not Supported")]
    NotSupported {},

    #[error("Can't derive owner from provided credentials")]
    NotDerivable {},

    #[error("Invalid signed action: {0}")]
    BadSignedAction(String),

    #[error("Account is frozen until ownership or credentials updates")]
    Frozen {},

    #[error("Deleted")]
    Deleted {},

    #[error("Provided credential is expired")]
    Expired {},

    #[error("Provided nonce has already been used")]
    NonceExists {},

    #[error("At least one of the provided credentials must be deriving into owner of the token")]
    NoOwnerCred {},

    #[error("At least one of the provided credentials must be must be usable for cryptographic verifications")]
    NoVerifyingCred {},

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
