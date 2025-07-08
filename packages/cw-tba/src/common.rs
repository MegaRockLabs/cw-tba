use crate::{msgs::*, Status};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, QuerierWrapper, StdError, StdResult};
use saa_wasm::StoredCredentials;
#[cfg(feature = "omniflix")]
use omniflix_std::types::omniflix::onft::v1beta1::{MsgTransferOnft,  OnftQuerier};


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
    if query_owner(querier, &token_info.collection, &token_info.id)?.owner != address {
        return Err(StdError::generic_err("Not NFT owner"));
    }
    Ok(())
}


#[cfg(not(feature = "omniflix"))]
fn query_owner(
    querier: &QuerierWrapper,
    collection: &str,
    token_id: &str,
) -> StdResult<OwnerOfResponse> {
    querier.query_wasm_smart(
        collection,
        &Cw721Msg::OwnerOf {
            token_id: token_id.to_string(),
            include_expired: None,
        },
    )
}

#[cfg(feature = "omniflix")]
fn query_owner(
    querier: &QuerierWrapper,
    denom: &str,
    token_id: &str,
) -> StdResult<OwnerOfResponse> {
    let res = OnftQuerier::new(querier).onft(denom.to_string(), token_id.to_string())?;
    if let Some(onft) = res.onft {
        Ok(OwnerOfResponse {
            owner: onft.owner,
            approvals: vec![],
        })
    } else {
        Err(StdError::generic_err("Token not found"))
    }
}

#[cfg(not(feature = "omniflix"))]
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


#[cfg(feature = "omniflix")]
pub fn query_tokens(
    querier: &QuerierWrapper,
    denom: &str,
    owner: String,
    _start_after: Option<String>,
    _limit: Option<u32>,
) -> StdResult<TokensResponse> {
    let res = OnftQuerier::new(querier).owner_onf_ts(
        denom.to_string(),
        owner,
        None
    )?;
    
    match res.owner {
        None => return Err(StdError::generic_err("Owner not found")),
        Some(o) => {
            return Ok(TokensResponse {
                tokens: o.id_collections
                        .into_iter()
                        .find_map(|id_collection| {
                            if id_collection.denom_id == denom {
                                Some(id_collection.onft_ids)
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default()
            });
        }
        
    }

}


#[cfg(feature = "omniflix")]
pub fn transfer_nft_msg(
    denom_id: &str,
    token_id: &str,
    sender: &str,
    recipient: &str,
) -> cosmwasm_std::CosmosMsg {

    MsgTransferOnft {
        denom_id: denom_id.to_string(),
        id: token_id.to_string(),
        sender: sender.to_string(),
        recipient: recipient.to_string(),
    }
    .into()
}


#[cfg(not(feature = "omniflix"))]
pub fn transfer_nft_msg(
    collection: &str,
    token_id: &str,
    _sender: &str,
    recipient: &str,
) -> cosmwasm_std::CosmosMsg {
    cosmwasm_std::CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
        contract_addr: collection.to_string(),
        msg: cosmwasm_std::to_json_binary(&Cw721Msg::TransferNft {
            recipient: recipient.to_string(),
            token_id: token_id.to_string(),
        }).unwrap_or_default(),
        funds: vec![],
    })

}


