use crate::{common::TokenInfo, ExecuteAccountMsg};
use cosmwasm_schema::{cw_serde, serde::Serialize};
use cosmwasm_std::{Binary, Coin, Empty};

#[cw_serde]
pub struct RegistryParams<T = Option<Empty>> {
    pub allowed_code_ids: Vec<u64>,
    pub creation_fees: Vec<Coin>,
    pub managers: Vec<String>,
    pub extension: T,
}

/// An extenstion for [cw83::CreateAccountMsg]
#[cw_serde]
pub struct TokenAccount<D = Binary, A = ExecuteAccountMsg>
where
    A: Serialize
{
    /// Non-Fungible Token Info that the created account will be linked to
    pub token_info: TokenInfo,

    /// Account data used for (cw81 signature verification)
    pub account_data: D,

    /// Actions to execute immediately on the account creation
    pub actions: Option<Vec<A>>,

    /// Optional parameter to create an account on behalf of another user that holds the token
    pub create_for: Option<String>,
}
