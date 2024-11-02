use cosmwasm_schema::{cw_serde, serde::Serialize, QueryResponses};
use cosmwasm_std::{Binary, Empty};
use cw83::{
    registry_query, AccountInfoResponse as AccountInfoResponseBase,
    AccountQuery as AccountQueryBase,
};

use cw_tba::{CreateAccountMsg, MigrateAccountMsg, RegistryParams, TokenInfo};

#[cw_serde]
pub struct InstantiateMsg {
    pub params: RegistryParams,
}

/// A List of the collections registered in the registry
#[cw_serde]
pub struct CollectionsResponse {
    /// Contract addresses of each collections
    pub collections: Vec<String>,
}

/// An full account stored in the registry
#[cw_serde]
pub struct Account {
    /// Token info of the account
    pub token_info: TokenInfo,

    /// Address of the token-bound account
    pub address: String,
}

/// An entry without collection address
#[cw_serde]
pub struct CollectionAccount {
    /// Token id
    pub token_id: String,
    /// Address of the token-bound account
    pub address: String,
}

#[cw_serde]
pub struct AccountsResponse {
    /// Total number of accounts in the registry
    pub total: u32,
    /// List of the accounts matching the query
    pub accounts: Vec<Account>,
}

#[cw_serde]
pub struct CollectionAccountsResponse {
    /// Total number of accounts of a specific collection
    pub total: u32,
    /// List of the accounts matching the query
    pub accounts: Vec<CollectionAccount>,
}

pub type AccountQuery = AccountQueryBase<TokenInfo>;
pub type AccountInfoResponse = AccountInfoResponseBase<Empty>;

#[registry_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Query all accounts in the registry in descending order
    #[returns(AccountsResponse)]
    Accounts {
        /// Number of accounts to skip
        /// [NOTE]: Not same as `start_after`
        skip: Option<u32>,
        /// Limit how many accounts to return
        limit: Option<u32>,
    },

    /// Query accounts linked to a token of a specific collection in descending order
    #[returns(CollectionAccountsResponse)]
    CollectionAccounts {
        /// Contract address of the collection
        collection: String,
        /// Number of accounts to skip
        skip: Option<u32>,
        /// Limit how many accounts to return
        limit: Option<u32>,
    },

    /// Query all the collections the registry is aware of
    #[returns(CollectionsResponse)]
    Collections {
        /// Number of collections to skip
        skip: Option<u32>,
        /// Limit how many collections to return
        limit: Option<u32>,
    },

    /// Query params of the registry
    #[returns(RegistryParams)]
    RegistryParams {},
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub enum ExecuteMsg<T = Binary>
where
    T: Serialize,
{
    CreateAccount(CreateAccountMsg<T>),

    /// Update the owner of a token-bound account
    UpdateAccountOwnership {
        /// Non-Fungible Token Info that the existing account is linked to
        token_info: TokenInfo,
        /// New data of the account used for (cw81 signature verification)
        new_account_data: Option<Binary>,
        /// Admin only parameter to update the account on behalf of another user that holds the token
        update_for: Option<String>,
    },

    /// Create a new token-bound account. Access the old one will be forever lost
    ResetAccount(CreateAccountMsg<T>),

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
