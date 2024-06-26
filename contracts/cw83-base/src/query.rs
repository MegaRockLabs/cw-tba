use cosmwasm_std::{Deps, Order, StdResult};
use cw_tba::TokenInfo;

use crate::{
    msg::{
        Account, AccountInfoResponse, AccountsResponse, CollectionAccount,
        CollectionAccountsResponse, CollectionsResponse,
    },
    state::{COL_TOKEN_COUNTS, TOKEN_ADDRESSES},
};

const DEFAULT_BATCH_SIZE: u32 = 100;

pub fn account_info(deps: Deps, info: TokenInfo) -> StdResult<AccountInfoResponse> {
    let address =
        TOKEN_ADDRESSES.load(deps.storage, (info.collection.as_str(), info.id.as_str()))?;
    Ok(AccountInfoResponse {
        address,
        info: None,
    })
}

pub fn collections(
    deps: Deps,
    skip: Option<u32>,
    limit: Option<u32>,
) -> StdResult<CollectionsResponse> {
    let skip = skip.unwrap_or(0) as usize;
    let limit = limit.unwrap_or(DEFAULT_BATCH_SIZE) as usize;

    let collections = COL_TOKEN_COUNTS
        .keys(deps.storage, None, None, Order::Descending)
        .into_iter()
        .enumerate()
        .filter(|(i, _)| *i >= skip)
        .take(limit)
        .map(|(_, c)| c.unwrap())
        .collect::<Vec<String>>();

    Ok(CollectionsResponse { collections })
}

pub fn accounts(deps: Deps, skip: Option<u32>, limit: Option<u32>) -> StdResult<AccountsResponse> {
    let skip = skip.unwrap_or(0) as usize;
    let limit = limit.unwrap_or(DEFAULT_BATCH_SIZE) as usize;

    let total = TOKEN_ADDRESSES
        .keys_raw(deps.storage, None, None, Order::Ascending)
        .count() as u32;

    let accounts = TOKEN_ADDRESSES
        .range(deps.storage, None, None, Order::Descending)
        .enumerate()
        .filter(|(i, _)| *i >= skip)
        .take(limit)
        .map(|(_, item)| {
            let ((collection, id), address) = item?;
            Ok(Account {
                token_info: TokenInfo { collection, id },
                address,
            })
        })
        .collect::<StdResult<Vec<Account>>>()?;

    Ok(AccountsResponse { accounts, total })
}

pub fn collection_accounts(
    deps: Deps,
    collection: &str,
    skip: Option<u32>,
    limit: Option<u32>,
) -> StdResult<CollectionAccountsResponse> {
    let skip = skip.unwrap_or(0) as usize;
    let limit = limit.unwrap_or(DEFAULT_BATCH_SIZE) as usize;

    let total = COL_TOKEN_COUNTS.load(deps.storage, collection)?;

    let accounts = TOKEN_ADDRESSES
        .prefix(collection)
        .range(deps.storage, None, None, Order::Descending)
        .enumerate()
        .filter(|(i, _)| *i >= skip)
        .take(limit)
        .map(|(_, item)| {
            let (id, address) = item?;
            Ok(CollectionAccount {
                token_id: id,
                address,
            })
        })
        .collect::<StdResult<Vec<CollectionAccount>>>()?;

    Ok(CollectionAccountsResponse { accounts, total })
}
