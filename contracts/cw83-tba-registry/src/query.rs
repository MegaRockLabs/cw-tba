use cosmwasm_std::{Deps, Order, StdResult};
use cw_tba::TokenInfo;

use crate::{
    msg::{Account, AccountOpt, Accounts},
    state::{COL_TOKEN_COUNTS, TOKEN_ADDRESSES},
};

const DEFAULT_BATCH_SIZE: u32 = 100;


pub fn account_info(deps: Deps, info: TokenInfo) -> StdResult<Account> {
    let info_key  = [info.collection.as_str(), info.id.as_str()];
    let address = TOKEN_ADDRESSES.load(deps.storage, info_key.into())?;

    Ok(Account {
        address,
        info,
    })
}


pub fn collections(
    deps: Deps,
    skip: Option<u32>,
    limit: Option<u32>,
) -> StdResult<Accounts> {
    let skip = skip.unwrap_or(0) as usize;
    let limit = limit.unwrap_or(DEFAULT_BATCH_SIZE) as usize;

    let iter = COL_TOKEN_COUNTS
        .keys(deps.storage, None, None, Order::Descending)
        .into_iter();

    let total = match iter.as_ref().size_hint() {
        (0, Some(total)) => total,
        (total, _) => total,
    } as u32;

    let collections = iter
        .enumerate()
        .filter(|(i, _)| *i >= skip)
        .take(limit)
        .map(|(_, c)| Ok(AccountOpt { info: None, address: c? }))
        .collect::<StdResult<Vec<AccountOpt>>>()?;

    Ok(Accounts { accounts: collections, total })
}

pub fn accounts(deps: Deps, skip: Option<u32>, limit: Option<u32>) -> StdResult<Accounts> {
    let skip = skip.unwrap_or(0) as usize;
    let limit = limit.unwrap_or(DEFAULT_BATCH_SIZE) as usize;


    let iter = TOKEN_ADDRESSES
        .range(deps.storage, None, None, Order::Descending);

    let total = match iter.as_ref().size_hint() {
        (0, Some(total)) => total,
        (total, _) => total,
    } as u32;

    let accounts = iter
        .enumerate()
        .filter(|(i, _)| *i >= skip)
        .take(limit)
        .map(|(_, item)| {
            let ((collection, id), address) = item?;
            Ok(AccountOpt {
                info: Some(TokenInfo { collection, id }),
                address,
            })
        })
        .collect::<StdResult<Vec<AccountOpt>>>()?;

    Ok(Accounts { accounts, total })
}


pub fn collection_accounts(
    deps: Deps,
    col: String,
    skip: Option<u32>,
    limit: Option<u32>,
) -> StdResult<Accounts> {
    let skip = skip.unwrap_or(0) as usize;
    let limit = limit.unwrap_or(DEFAULT_BATCH_SIZE) as usize;

    let iter = TOKEN_ADDRESSES
        .prefix(col.as_str())
        .range(deps.storage, None, None, Order::Descending);

    let total = match iter.as_ref().size_hint() {
        (0, Some(total)) => total,
        (total, _) => total,
    } as u32;

    let accounts = iter
        .enumerate()
        .filter(|(i, _)| *i >= skip)
        .take(limit)
        .map(|(_, item)| {
            let (id, address) = item?;
            Ok(AccountOpt { address, info: Some(TokenInfo { collection: col.clone(), id })})
        })
        .collect::<StdResult<Vec<AccountOpt>>>()?;

    Ok(Accounts { accounts, total })
}
