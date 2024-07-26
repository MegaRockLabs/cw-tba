use cosmwasm_schema::{cw_serde, serde::Serialize, QueryResponses};
use cosmwasm_std::{Binary, Coin, CosmosMsg, Empty};
use cw82::smart_account_query;
use cw_ownable::cw_ownable_query;
use schemars::JsonSchema;

use crate::common::TokenInfo;
use cw721::Cw721ReceiveMsg;

#[cw_serde]
pub struct InstantiateAccountMsg<T = Binary>
where
    T: Serialize,
{
    /// Token owner that had been verified by the registry
    pub owner: String,
    /// Token info
    pub token_info: TokenInfo,
    /// Customiable payload specififc for account implementation
    pub account_data: T,
}

#[cw_serde]
pub struct MigrateAccountMsg<T = Empty> {
    pub params: Box<Option<T>>,
}


#[cw_serde]
pub struct Status {
    /// Whether the account is frozen
    pub frozen: bool,
}



#[cw_serde]
pub enum ExecuteAccountMsg<T = Empty, E = Option<Empty>, A = Binary> {
    /// Proxy method for executing cosmos messages
    /// Wasm and Stargate messages aren't supported
    /// Only the current holder can execute this method
    Execute { msgs: Vec<CosmosMsg<T>> },
    /// Mint NFTs directly from token account
    MintToken {
        /// Contract address of the minter
        minter: String,
        // Mint message to pass a minter contract
        msg: Binary,
    },
    /// Send NFT to a contract
    SendToken {
        /// Contract address of the collection
        collection: String,
        /// Token id
        token_id: String,
        /// Recipient contract address
        contract: String,
        /// Send message to pass a recipient contract
        msg: Binary,
    },
    /// Simple NFT transfer
    TransferToken {
        /// Contract address of the collection
        collection: String,
        /// Token id
        token_id: String,
        /// Recipient address
        recipient: String,
    },
    /// Owner only method to make the account forget about certain tokens
    ForgetTokens {
        /// Contract address of the collection
        collection: String,
        /// Optional list of token ids to forget. If not provided, all tokens will be forgotten
        token_ids: Vec<String>,
    },

    /// Owner only method that make the account aware of certain tokens to simplify the future queries
    UpdateKnownTokens {
        /// Contract address of the collection
        collection: String,
        /// Token id to start after
        start_after: Option<String>,
        /// Limit of the tokens to return
        limit: Option<u32>,
    },

    /// Registry only method to update the owner to the current NFT holder
    UpdateOwnership {
        /// Current NFT holder
        new_owner: String,
        /// New account data
        new_account_data: Option<A>,
    },

    /// Owner only method to update account data
    UpdateAccountData {
        /// New account data
        new_account_data: A,
    },

    /// Registering a token as known on receiving
    ReceiveNft(Cw721ReceiveMsg),

    /// Registry only method to call when a token is moved to escrow
    Freeze {},

    /// Registry only method to call after the token is released from escrow
    Unfreeze {},

    /// Remove all the data from the contract and make it unsuable
    Purge {},

    /// Extension
    Extension { msg: E },
}


pub type KnownTokensResponse = Vec<TokenInfo>;


#[cw_serde]
pub struct AssetsResponse {
    /// Native fungible tokens held by an account
    pub balances: Vec<Coin>,
    /// NFT tokens the account is aware of
    pub tokens: Vec<TokenInfo>,
}




#[smart_account_query]
#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryAccountMsg<T = Empty, Q: JsonSchema = Empty> {
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

    /// Incremental number telling wether a direct interaction with the account has occured
    #[returns(u128)]
    Serial {},

    #[returns(())]
    Extension { msg: Q },
}