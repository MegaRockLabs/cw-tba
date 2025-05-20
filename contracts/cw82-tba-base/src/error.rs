use cosmwasm_std::StdError;
use cw_ownable::OwnershipError;
use cw_utils::ParseReplyError;
use cw_auths::saa_types::{AuthError, SessionError};
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

    #[error("Session {0}")]
    Session(#[from] SessionError),
    

    #[error("Passed Credential data must have only one Secpl251 credential")]
    PubkeyOnly {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Not Supported")]
    NotSupported {},

    #[error("Deleted")]
    Deleted {},

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
