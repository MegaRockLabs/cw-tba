use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Timestamp};
use crate::TokenAccount;


pub type CreateAccountMsg<T> = cw83::CreateAccountMsg<TokenAccount<T>>;


#[cw_serde]
#[derive(Copy)]
pub enum Expiration {
    /// AtHeight will expire when `env.block.height` >= height
    AtHeight(u64),
    /// AtTime will expire when `env.block.time` >= time
    AtTime(Timestamp),
    /// Never will never expire. Used to express the empty variant
    Never {},
}


#[cw_serde]
pub struct Approval {
    pub spender: String,
    pub expires: Expiration,
}

#[cw_serde]
pub enum Cw721Msg {
    
    TransferNft { 
        recipient: String, 
        token_id: String 
    },
 
    SendNft {
        contract: String,
        token_id: String,
        msg: Binary,
    },

    OwnerOf {
        token_id: String,
        include_expired: Option<bool>,
    },

    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    }
} 


#[cw_serde]
pub struct Cw721ReceiveMsg {
    pub sender: String,
    pub token_id: String,
    pub msg: Binary,
}



#[cw_serde]
pub struct OwnerOfResponse {
    pub owner: String,
    pub approvals: Vec<Approval>,
}


#[cw_serde]
pub struct TokensResponse {
    pub tokens: Vec<String>,
}
