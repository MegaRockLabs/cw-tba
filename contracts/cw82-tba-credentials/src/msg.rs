use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Empty, Response};
use cw_tba::{ExecuteAccountMsg, InstantiateAccountMsg, MigrateAccountMsg, QueryAccountMsg, Status, TokenInfo};
use saa::{messages::{AccountCredentials, SignedData}, CredentialData};

use crate::error::ContractError;




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





pub type InstantiateMsg = InstantiateAccountMsg;
pub type ExecuteMsg = ExecuteAccountMsg<SignedData<ExecuteAccountMsg>, Option<Empty>, CredentialData>;

pub type MigrateMsg = MigrateAccountMsg;
pub type ContractResult = Result<Response, ContractError>;

pub type QueryMsg = QueryAccountMsg<SignedData<ExecuteAccountMsg>, CredQueryMsg>;


#[cw_serde]
pub enum SudoMsg {
    #[cfg(feature = "archway")]
    CwGrant(crate::grants::CwGrant)
}