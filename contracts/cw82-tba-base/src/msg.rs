use cosmwasm_schema::{cw_serde, serde::Serialize};
use cosmwasm_std::{Addr, Binary, Coin, Empty};
pub use cw82::{
    account_query, CanExecuteResponse
};
use cw_ownable::cw_ownable_query;
use cw_tba::{InstantiateAccountMsg, MigrateAccountMsg, TokenInfo};
use saa_schema::QueryResponses;

pub type InstantiateMsg = InstantiateAccountMsg;
pub type MigrateMsg = MigrateAccountMsg;



#[cw_serde]
pub struct Status {
    /// Whether the account is frozen
    pub frozen: bool,
}

#[cw_serde]
pub struct AssetsResponse {
    /// Native fungible tokens held by an account
    pub balances: Vec<Coin>,
    /// NFT tokens the account is aware of
    pub tokens: Vec<TokenInfo>,
}

#[cw_serde]
pub struct FullInfoResponse {
    /// Current owner of the token account that is ideally a holder of an NFT
    pub ownership: cw_ownable::Ownership<Addr>,
    /// Public key that is used to verify signed messages
    pub pubkey: Binary,
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
}

pub type KnownTokensResponse = Vec<TokenInfo>;


#[account_query]
#[cw_ownable_query]
#[derive(QueryResponses)]
#[cw_serde]
pub enum QueryMsgBase<T : Serialize + Clone = Empty> {
    /// Public key that is used to verify signed messages
    #[returns(Binary)]
    Pubkey {},

    /// Status of the account telling whether it iz frozen
    #[returns(Status)]
    Status {},

    /// NFT token the account is linked to
    #[returns(TokenInfo)]
    Token {},

    /// Registry address
    #[returns(String)]
    Registry {},

    /// List of the tokens the account is aware of
    #[returns(KnownTokensResponse)]
    KnownTokens {
        skip: Option<u32>,
        limit: Option<u32>,
    },

    /// List of the assets (balances + tokens) the account is aware of
    #[returns(AssetsResponse)]
    Assets {
        skip: Option<u32>,
        limit: Option<u32>,
    },

    /// Full info about the account
    #[returns(FullInfoResponse)]
    FullInfo {
        skip: Option<u32>,
        limit: Option<u32>,
    },

    /// Incremental number telling wether a direct interaction with the account has occured
    #[returns(u128)]
    Serial {},

}

/// [TokenInfo] is used as a to query the account info
/// so no need to return any additional data
pub type QueryMsg = QueryMsgBase<Empty>;
