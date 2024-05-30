use cosmwasm_std::{Addr, Binary, Coin, CosmosMsg, CustomMsg, Empty, Response, Timestamp};
use cosmwasm_schema::{cw_serde, schemars::JsonSchema, QueryResponses};
pub use cw82::{
    smart_account_query, 
    CanExecuteResponse, 
    ValidSignatureResponse, 
    ValidSignaturesResponse
};
use cw_ownable::cw_ownable_query;
use cw_tba::{TokenInfo, InstantiateAccountMsg, ExecuteAccountMsg, MigrateAccountMsg};
use saa::{CredentialData, CredentialId};

use crate::error::ContractError;



#[cw_serde]
pub struct AuthPayload {
    pub hrp:              Option<String>,
    pub address:          Option<String>,
    pub credential_id:    Option<CredentialId>,
}


#[cw_serde]
pub struct IndexedAuthPayload {
    pub payload: AuthPayload,
    pub index: u8
}


#[cw_serde]
pub enum ValidSignaturesPayload {
    Generic(AuthPayload),
    Multiple(Vec<Option<IndexedAuthPayload>>)
}


#[cw_serde]
pub struct CosmosMsgDataToSign {
    pub messages   :  Vec<CosmosMsg<Empty>>,
    pub chain_id   :  String,
    pub timestamp  :  Timestamp
}

impl CustomMsg for CosmosMsgDataToSign {}


#[cw_serde]
pub struct AccountActionDataToSign {
    pub actions    :  Vec<ExecuteAccountMsg>,
    pub chain_id   :  String,
    pub timestamp  :  Timestamp
}


#[cw_serde]
pub struct SignedCosmosMsgs {
    pub data        : CosmosMsgDataToSign,
    pub payload     : Option<AuthPayload>,
    pub signature   : Binary,
}


#[cw_serde]
pub struct SignedAccountActions {
    pub data        : AccountActionDataToSign,
    pub payload     : Option<AuthPayload>,
    pub signature   : Binary,
}


impl CustomMsg for SignedCosmosMsgs {}
impl CustomMsg for SignedAccountActions {}


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
    pub tokens: Vec<TokenInfo>
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
    pub status: Status
}


pub type KnownTokensResponse = Vec<TokenInfo>;



#[smart_account_query]
#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsgBase <T = SignedCosmosMsgs, Q: JsonSchema = Empty> {

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
        limit: Option<u32>
    },

    /// List of the assets (balances + tokens) the account is aware of
    #[returns(AssetsResponse)]
    Assets {
        skip: Option<u32>,
        limit: Option<u32>
    },

    /// Full info about the account
    #[returns(FullInfoResponse)]
    FullInfo {
        skip: Option<u32>,
        limit: Option<u32>
    },

    /// Incremental number telling wether a direct interaction with the account has occured
    #[returns(u128)]
    Serial {},

    #[returns(())]
    Extension { msg: Q }
}

/// [TokenInfo] is used as a to query the account info
/// so no need to return any additional data



pub type ContractResult = Result<Response, ContractError>;
pub type InstantiateMsg = InstantiateAccountMsg<CredentialData>;
pub type ExecuteMsg = ExecuteAccountMsg<SignedCosmosMsgs, SignedAccountActions, CredentialData>;
pub type QueryMsg = QueryMsgBase<SignedCosmosMsgs, Empty>;
pub type MigrateMsg = MigrateAccountMsg;
