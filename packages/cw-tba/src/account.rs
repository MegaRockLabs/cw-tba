use cosmwasm_schema::{cw_serde, serde::Serialize};
use cosmwasm_std::{Binary, CosmosMsg, Empty};

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
