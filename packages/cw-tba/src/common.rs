use crate::{msgs::*, Status};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, QuerierWrapper, StdError, StdResult};
use saa_wasm::StoredCredentials;

#[cw_serde]
pub struct TokenInfo {
    /// Contract address of the collection
    pub collection: String,
    /// Token id
    pub id: String,
}

impl TokenInfo {
    pub fn key(&self) -> (&str, &str) {
        (self.collection.as_str(), self.id.as_str())
    }
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
    pub credentials: StoredCredentials,
}

pub fn verify_nft_ownership(
    querier: &QuerierWrapper,
    address: &str,
    token_info: TokenInfo,
) -> StdResult<()> {
    if querier.query_wasm_smart::<OwnerOfResponse>(
        token_info.collection,
        &Cw721Msg::OwnerOf {
            token_id: token_info.id,
            include_expired: None,
        },
    )?.owner != address {
        return Err(StdError::generic_err("Not NFT owner"));
    }
    Ok(())
}


pub fn query_tokens(
    querier: &QuerierWrapper,
    collection: &str,
    owner: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<TokensResponse> {
    querier.query_wasm_smart(
        collection,
        &Cw721Msg::Tokens {
            owner,
            start_after,
            limit,
        },
    )
}
