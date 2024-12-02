use anybuf::Anybuf;
use cosmwasm_schema::{cw_serde, serde::Serialize, QueryResponses};
use cosmwasm_std::{AnyMsg, Binary, Coin, CosmosMsg, Empty, StdResult, Timestamp};
use cw82::smart_account_query;
use cw_ownable::cw_ownable_query;
use schemars::JsonSchema;
pub use saa::UpdateOperation;

use crate::common::TokenInfo;
use crate::Cw721ReceiveMsg;

#[cw_serde]
pub struct InstantiateAccountMsg<A = ExecuteAccountMsg, T = Binary>
where
    T: Serialize,
{
    /// Customiable payload specififc for account implementation
    pub account_data: T,
    /// Actions to execute immediately on the account creation
    pub actions: Option<Vec<A>>,
    /// Token info
    pub token_info: TokenInfo,
    /// Token owner that had been verified by the registry
    pub owner: String,
}


#[cw_serde]
pub struct MigrateAccountMsg<T = Empty> {
    pub params: Option<Box<T>>,
}


#[cw_serde]
pub struct Status {
    /// Whether the account is frozen
    pub frozen: bool,
}


#[cw_serde]
pub struct BasicAllowance {
    pub expiration  : Option<Timestamp>,
    pub spend_limit : Vec<Coin>,
}


#[cw_serde]
pub enum ExecuteAccountMsg<T = Empty, A : Serialize = Binary, E = Option<Empty>> {
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
        /// New NFT holder
        new_owner: String,
        /// New account data
        new_account_data: Option<A>,
    },

    /// Owner only method to update account data
    UpdateAccountData {
        /// Old data to proof ownership
        account_data: Option<A>,
        /// New account data
        operation: UpdateOperation<A>,
    },

    /// Registering a token as known on receiving
    ReceiveNft(Cw721ReceiveMsg),

    FeeGrant {
        grantee     :   String,
        allowance   :   Option<BasicAllowance>

    },

    /// Registry only method to call when a token is moved to escrow
    Freeze {},

    /// Registry only method to call after the token is released from escrow
    Unfreeze {},

    /// Remove all the data from the contract and make it unsuable
    Purge {},

    /// Extension
    Extension { msg: E },
}


impl Default for ExecuteAccountMsg {
    fn default() -> Self {
        ExecuteAccountMsg::Execute { msgs: vec![] }
    }
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





pub fn encode_feegrant_msg(
    granter       : &str, 
    grantee       : &str,
    allowance     : Option<BasicAllowance>,
) -> StdResult<CosmosMsg> {

    let (spend_limit, expiration) = match allowance {
        Some(allowance) => {
            let coins  = allowance.spend_limit
                .iter()
                .map(|coin| Anybuf::new() 
                    .append_string(1, &coin.denom)
                    .append_string(2, &coin.amount.to_string())
                )
                .collect();

            let expiration = allowance.expiration
                .map(|ts| Anybuf::new()
                    .append_int64(1, ts.seconds() as i64)
                    .append_int32(2, 0i32)
                );
            
            (coins, expiration)

        },
        None => (vec![], None),
    };

    let mut basic_msg = Anybuf::new()
        .append_repeated_message(1, &spend_limit);
    
    if expiration.is_some() {
        basic_msg = basic_msg.append_message(2, &expiration.unwrap());
    }

    let basic  = Anybuf::new()
        .append_string(1, "/cosmos.feegrant.v1beta1.BasicAllowance")
        .append_message(2, &basic_msg);


    let allowed_msg = Anybuf::new()
        .append_string(1, "/cosmos.feegrant.v1beta1.AllowedMsgAllowance")
        .append_message(2,&Anybuf::new()
            .append_message(1, &basic)
            .append_repeated_string(2, &["/cosmwasm.wasm.v1.MsgExecuteContract"])
        );


    let msg : CosmosMsg = CosmosMsg::Any(AnyMsg {
        type_url: "/cosmos.feegrant.v1beta1.MsgGrantAllowance".to_string(),
        value: anybuf::Anybuf::new()
                .append_string(1, granter)
                .append_string(2, grantee)
                .append_message(3, &allowed_msg)
                .into_vec()
                .into()
    });


    Ok(msg)
}

