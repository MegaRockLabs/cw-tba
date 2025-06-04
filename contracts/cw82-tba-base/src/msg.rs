use cosmwasm_schema::{cw_serde, serde::Serialize};
use cosmwasm_std::{Addr, Binary, Coin, Empty};
use cw82::smart_account_query_t;
pub use cw82::{
    smart_account_query, CanExecuteResponse, ValidSignatureResponse, ValidSignaturesResponse,
};
use cw_ownable::cw_ownable_query;
use cw_tba::{ExecuteAccountMsg, InstantiateAccountMsg, MigrateAccountMsg, TokenInfo};
use saa_wasm::session_query;

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


#[smart_account_query_t]
#[cw_ownable_query]
#[session_query(ExecuteAccountMsg)]
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
