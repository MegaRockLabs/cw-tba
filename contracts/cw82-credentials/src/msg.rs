use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Coin, CustomMsg, Empty, Response, Uint128};
pub use cw82::{smart_account_query, CanExecuteResponse, ValidSignatureResponse, ValidSignaturesResponse,};
use cw_tba::{ExecuteAccountMsg, InstantiateAccountMsg, MigrateAccountMsg, QueryAccountMsg, Status, TokenInfo};
use saa::{CredentialData, CredentialId};

use crate::error::ContractError;

#[cw_serde]
pub struct AuthPayload {
    pub hrp: Option<String>,
    pub address: Option<String>,
    pub credential_id: Option<CredentialId>,
}

#[cw_serde]
pub struct IndexedAuthPayload {
    pub payload: AuthPayload,
    pub index: u8,
}

#[cw_serde]
pub enum ValidSignaturesPayload {
    Generic(AuthPayload),
    Multiple(Vec<Option<IndexedAuthPayload>>),
}


#[cw_serde]
pub struct ActionDataToSign {
    pub chain_id: String,
    pub messages: Vec<ExecuteAccountMsg>,
    pub nonce: Uint128,
}


#[cw_serde]
pub struct SignedActions {
    pub data: ActionDataToSign,
    pub payload: Option<AuthPayload>,
    pub signature: Binary,
}

impl CustomMsg for SignedActions {}



#[cw_serde]
pub struct CredentialFullInfo {
    pub id: CredentialId,
    pub name: String,
    pub hrp: Option<String>,
}

#[cw_serde]
pub struct AccountCredentials {
    pub credentials: Vec<CredentialFullInfo>,
    pub verifying_id: CredentialId,
    pub native_caller: bool,
}


#[cw_serde]
pub enum CredQueryMsg {
    FullInfo {
        skip: Option<u32>,
        limit: Option<u32>,
    },
    Credentials {},
}


#[cw_serde]
pub struct FullInfoResponse {
    /// Current owner of the token account that is ideally a holder of an NFT
    pub ownership: cw_ownable::Ownership<Addr>,
    /// Token info
    pub token_info: TokenInfo,
    /// Registry address
    pub registry: String,
    /// Native fungible tokens held by an account
    pub balances: Vec<Coin>,
    /// NFT tokens the account is aware of
    pub tokens: Vec<TokenInfo>,
    /// Whether the account is frozen
    pub status: Status,
    /// Full info about installed credentials
    pub credentials: AccountCredentials
}



/// [TokenInfo] is used as a to query the account info
/// so no need to return any additional data

pub type InstantiateMsg = InstantiateAccountMsg<Binary>;
pub type ExecuteMsg = ExecuteAccountMsg<SignedActions, Empty, CredentialData>;

pub type MigrateMsg = MigrateAccountMsg;
pub type ContractResult = Result<Response, ContractError>;

pub type QueryMsg = QueryAccountMsg<SignedActions, CredQueryMsg>;