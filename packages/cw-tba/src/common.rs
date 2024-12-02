use cosmwasm_schema::cw_serde;
use cosmwasm_std::{QuerierWrapper, StdError, StdResult};

use crate::msgs::*;

#[cw_serde]
pub struct TokenInfo {
    /// Contract address of the collection
    pub collection: String,
    /// Token id
    pub id: String,
}



pub fn verify_nft_ownership(
    querier: &QuerierWrapper,
    address: &str,
    token_info: TokenInfo,
) -> StdResult<()> {
    let owner_res = querier.query_wasm_smart::<OwnerOfResponse>(
        token_info.collection,
        &Cw721Msg::OwnerOf {
            token_id: token_info.id,
            include_expired: None,
        },
    )?;

    if owner_res.owner.as_str() != address {
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
