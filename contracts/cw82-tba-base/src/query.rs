use cosmwasm_std::{Addr, Binary, Deps, Env, Order, StdError, StdResult};
use cw82::{CanExecuteResponse, ValidSignatureResponse};
use cw_ownable::is_owner;
use cw_tba::TokenInfo;
use smart_account_auth::{CosmosArbitrary, Credential, Verifiable};

use crate::{
    msg::{AssetsResponse, FullInfoResponse},
    state::{KNOWN_TOKENS, PUBKEY, REGISTRY_ADDRESS, STATUS, TOKEN_INFO},
    utils::{assert_ok_cosmos_msg, assert_status, status_ok},
};

const DEFAULT_BATCH_SIZE: u32 = 100;

pub fn can_execute(
    deps: Deps,
    sender: String,
    msg: &cosmwasm_std::CosmosMsg,
) -> StdResult<CanExecuteResponse> {
    let cant = CanExecuteResponse { can_execute: false };

    if !status_ok(deps.storage) {
        return Ok(cant);
    };

    let addr_validity = deps.api.addr_validate(&sender);
    if addr_validity.is_err() {
        return Ok(cant);
    };

    let res = is_owner(deps.storage, &addr_validity.unwrap());
    if res.is_err() || !res.unwrap() {
        return Ok(cant);
    };

    Ok(CanExecuteResponse {
        can_execute: assert_ok_cosmos_msg(msg).is_ok(),
    })
}

pub fn valid_signature(
    deps: Deps,
    data: Binary,
    signature: Binary,
    _payload: &Option<Credential>,
) -> StdResult<ValidSignatureResponse> {
    let pk: Binary = PUBKEY.load(deps.storage)?;
    let owner = cw_ownable::get_ownership(deps.storage)?;

    let address = owner.owner.unwrap_or(Addr::unchecked(""));

    Ok(ValidSignatureResponse {
        is_valid: assert_status(deps.storage).is_ok() &&
            verify_arbitrary(deps, address.as_str(), data, signature, &pk).is_ok()
    })
}

pub fn verify_arbitrary(
    deps: Deps,
    account_addr: &str,
    message: Binary,
    signature: Binary,
    pubkey: &[u8],
) -> StdResult<bool> {
    let arb = CosmosArbitrary {
        pubkey: pubkey.into(),
        signature,
        message,
        address: account_addr.to_string(),
    };

    arb.verify(deps)
        .map_err(|_| StdError::generic_err("Invalid signature"))?;

    Ok(true)
}

pub fn assets(
    deps: Deps,
    env: Env,
    skip: Option<u32>,
    limit: Option<u32>,
) -> StdResult<AssetsResponse> {
    let nfts = known_tokens(deps, skip, limit)?;
    let balance = deps.querier.query_all_balances(env.contract.address)?;

    Ok(AssetsResponse {
        balances: balance,
        tokens: nfts,
    })
}

pub fn known_tokens(
    deps: Deps,
    skip: Option<u32>,
    limit: Option<u32>,
) -> StdResult<Vec<TokenInfo>> {
    let skip = skip.unwrap_or(0) as usize;
    let limit = limit.unwrap_or(DEFAULT_BATCH_SIZE) as usize;

    let tokens: StdResult<Vec<TokenInfo>> = KNOWN_TOKENS
        .keys(deps.storage, None, None, Order::Ascending)
        .enumerate()
        .filter(|(i, _)| *i >= skip)
        .take(limit)
        .map(|(_, kt)| {
            let kp = kt?;
            Ok(TokenInfo {
                collection: kp.0,
                id: kp.1,
            })
        })
        .collect();

    tokens
}

pub fn full_info(
    deps: Deps,
    env: Env,
    skip: Option<u32>,
    limit: Option<u32>,
) -> StdResult<FullInfoResponse> {
    let tokens = known_tokens(deps, skip, limit)?;
    let balances = deps.querier.query_all_balances(env.contract.address)?;
    let ownership = cw_ownable::get_ownership(deps.storage)?;

    Ok(FullInfoResponse {
        balances,
        tokens,
        ownership,
        registry: REGISTRY_ADDRESS.load(deps.storage)?,
        pubkey: PUBKEY.load(deps.storage)?,
        token_info: TOKEN_INFO.load(deps.storage)?,
        status: STATUS.load(deps.storage)?,
    })
}
