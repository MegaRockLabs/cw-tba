use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;
use cw_auths::saa_types::{msgs::SignedDataMsg, Expiration};
use crate::TokenAccount;


pub type CreateAccountMsg = cw83::CreateAccountMsg<TokenAccount>;


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


pub type CosmosMsg = cosmwasm_std::CosmosMsg<SignedDataMsg>;