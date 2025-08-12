use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Coin};
pub use cw82::{account_query, CanExecuteResponse};
pub use cw_tba::{InstantiateAccountMsg as InstantiateMsg, QueryMsg, TokenInfo};

pub type MigrateMsg = Binary;

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

