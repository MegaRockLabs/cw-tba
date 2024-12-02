use cosmwasm_schema::schemars;
use cosmwasm_std::{Addr, Coin, Response};
use cw_tba::{ExecuteAccountMsg, InstantiateAccountMsg, MigrateAccountMsg, QueryAccountMsg, Status, TokenInfo};
use saa::{messages::{AccountCredentials, SignedDataMsg}, CredentialData};
use crate::error::ContractError;



#[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CredQueryMsg {
    FullInfo {
        skip: Option<u32>,
        limit: Option<u32>,
    },
    Credentials {},
}


#[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SignedMessages {
    pub messages: Vec<ExecuteAccountMsg>
}


#[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum SudoMsg {
    #[cfg(feature = "archway")]
    CwGrant(crate::grants::CwGrant)
}


pub type InstantiateMsg = InstantiateAccountMsg;
pub type ExecuteMsg = ExecuteAccountMsg<SignedDataMsg, CredentialData>;

pub type MigrateMsg = MigrateAccountMsg;
pub type ContractResult = Result<Response, ContractError>;

pub type QueryMsg = QueryAccountMsg<SignedDataMsg, CredQueryMsg>;


