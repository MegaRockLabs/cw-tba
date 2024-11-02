use cosmwasm_schema::cw_serde;
use cosmwasm_std::{QuerierWrapper, StdError, StdResult, Uint128};

#[cw_serde]
pub struct TokenInfo {
    /// Contract address of the collection
    pub collection: String,
    /// Token id
    pub id: String,
}


#[cw_serde]
pub struct InitSignedMessage {
    /// chain id of the chain where the account contract is located
    pub chain_id: String,
    /// text to prompt the user to sign
    pub message: String,
    /// a unique number never used before. Can use 0 for first time
    pub nonce: Uint128,
    /// registy contract address
    pub registry: String
}




impl TokenInfo {
    pub fn key_tuple(&self) -> (&str, &str) {
        (self.collection.as_str(), &self.id.as_str())
    }
}

pub fn verify_nft_ownership(
    querier: &QuerierWrapper,
    address: &str,
    token_info: TokenInfo,
) -> StdResult<()> {
    let owner_res = querier.query_wasm_smart::<cw721::OwnerOfResponse>(
        token_info.collection,
        &cw721::Cw721QueryMsg::OwnerOf {
            token_id: token_info.id,
            include_expired: None,
        },
    )?;

    if owner_res.owner.as_str() != address {
        return Err(StdError::generic_err("Unauthorized"));
    }

    Ok(())
}

pub fn query_tokens(
    querier: &QuerierWrapper,
    collection: &str,
    owner: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<cw721::TokensResponse> {
    querier.query_wasm_smart(
        collection,
        &cw721::Cw721QueryMsg::Tokens {
            owner,
            start_after,
            limit,
        },
    )
}
