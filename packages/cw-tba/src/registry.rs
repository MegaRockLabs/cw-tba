use crate::{common::TokenInfo, ActiontMsg};
use smart_account_auth::CredentialData;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;


#[cw_serde]
pub struct RegistryParams {
    pub allowed_code_ids: Vec<u64>,
    pub creation_fees: Vec<Coin>,
    pub managers: Vec<String>,
}

/// An extenstion for [cw83::CreateAccountMsg]
#[cw_serde]
pub struct TokenAccountPayload {
    /// Non-Fungible Token Info that the created account will be linked to
    pub token_info: TokenInfo,

    /// Account data used for (cw81 signature verification)
    pub credential_data: CredentialData,

    /// Actions to execute immediately on the account creation
    pub actions: Option<Vec<ActiontMsg>>,

    /// Optional parameter to create an account on behalf of another user that holds the token
    pub create_for: Option<String>,
}
