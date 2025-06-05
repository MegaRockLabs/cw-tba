use cosmwasm_schema::{cw_serde, QueryResponses};
use cw83::{
    registry_execute, registry_query, AccountResponse, AccountsResponse
};

use cw_tba::{CreateAccountMsg, MigrateAccountMsg, RegistryParams, TokenInfo, TokenAccount};
use saa_wasm::{saa_types::{msgs::SignedDataMsg, CredentialData}, UpdateOperation};

#[cw_serde]
pub struct InstantiateMsg {
    pub params: RegistryParams,
}

#[cw_serde]
pub enum AccountsQueryMsg {
    Collection(String),
    Collections{}
}



#[allow(dead_code, unused)]
type OptTokenInfo       = Option<TokenInfo>;
type OptAccountsQuery   = Option<AccountsQueryMsg>;

/// An full account stored in the registry
pub type Account        = AccountResponse<TokenInfo>;
pub type AccountOpt     = AccountResponse<Option<TokenInfo>>;
pub type Accounts       = AccountsResponse<Option<TokenInfo>>;



#[registry_query(TokenInfo, TokenInfo, OptAccountsQuery, OptTokenInfo)]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Query params of the registry
    #[returns(RegistryParams)]
    RegistryParams {},
}




#[registry_execute(TokenAccount)]
#[cw_serde]
pub enum ExecuteMsg {

    /// Update the owner of a token-bound account
    UpdateAccountOwnership {
        /// Non-Fungible Token Info that the existing account is linked to
        token_info: TokenInfo,
        /// New data of the account used for (cw81 signature verification)
        new_account_data: Option<CredentialData>,
        /// Admin only parameter to update the account on behalf of another user that holds the token
        update_for: Option<String>,
    },

    UpdateAccountData {
        /// Non-Fungible Token Info that the existing account is linked to
        token_info: TokenInfo,
        /// New data on the account
        update_op: UpdateOperation,
        /// Signed information to update the account
        signed: Option<SignedDataMsg>,
    },

    /// Create a new token-bound account. Access the old one will be forever lost
    ResetAccount(CreateAccountMsg),

    /// Migrate an account to the newer code version if the code id is allowed
    MigrateAccount {
        /// Non-Fungible Token Info that the existing account is linked to
        token_info: TokenInfo,
        /// New code id to migrate the account to
        new_code_id: u64,
        /// Migration message to be passed to the account contract
        msg: MigrateAccountMsg,
    },
}

#[cw_serde]
pub enum SudoMsg {
    /// updating the entire registry params object
    UpdateParams(Box<RegistryParams>),
    /// updating an address that is used for fair fee burning
    UpdateFairBurnAddress(String),
    /// updating the list of code ids that are allowed for account creation & migration
    UpdateAllowedCodeIds { code_ids: Vec<u64> },
    /// manager contracts that can update an owner for an account if the latter is the new holder of the bound NFT
    UpdateManagers { managers: Vec<String> },
}



#[cw_serde]
pub struct MigrateMsg {}
