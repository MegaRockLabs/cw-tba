use cosmwasm_std::{from_json, Binary, CosmosMsg, Deps, Env, Order, StdError, StdResult};
use cw82::{CanExecuteResponse, ValidSignatureResponse, ValidSignaturesResponse};
use cw_tba::TokenInfo;
use saa::{ensure, Verifiable};

use crate::{
    msg::{AssetsResponse, FullInfoResponse, SignedCosmosMsgs, ValidSignaturesPayload},
    state::{KNOWN_TOKENS, REGISTRY_ADDRESS, STATUS, TOKEN_INFO},
    utils::{
        assert_signed_msg, assert_simple_msg, get_verifying_credential,
        get_verifying_indexed_credential, status_ok, validate_multi_payload,
    },
};

const DEFAULT_BATCH_SIZE: u32 = 100;

pub fn can_execute(
    deps: Deps,
    env: Env,
    sender: String,
    msg: CosmosMsg<SignedCosmosMsgs>,
) -> StdResult<CanExecuteResponse> {
    if !status_ok(deps.storage) {
        return Ok(CanExecuteResponse { can_execute: false });
    };

    let can_execute = match msg {
        CosmosMsg::Custom(signed) => assert_signed_msg(deps, &env, &sender, &signed).is_ok(),
        _ => assert_simple_msg(deps, &env, &sender, &msg).is_ok(),
    };

    Ok(CanExecuteResponse { can_execute })
}

pub fn valid_signature(
    deps: Deps,
    env: Env,
    data: Binary,
    signature: Binary,
    payload: Option<Binary>,
) -> StdResult<ValidSignatureResponse> {
    let is_valid = if status_ok(deps.storage) {
        let credential = get_verifying_credential(deps, data, signature, payload)?;
        credential.verified_cosmwasm(deps.api, &env, &None).is_ok()
    } else {
        false
    };

    Ok(ValidSignatureResponse { is_valid })
}

pub fn valid_signatures(
    deps: Deps,
    env: Env,
    data: Vec<Binary>,
    signatures: Vec<Binary>,
    payload: Option<Binary>,
) -> StdResult<ValidSignaturesResponse> {
    if !status_ok(deps.storage) {
        return Ok(ValidSignaturesResponse {
            are_valid: vec![false, data.len() > 1],
        });
    };

    ensure!(
        data.len() == signatures.len(),
        StdError::generic_err("Data and signatures must be of equal length")
    );

    let payload = if payload.is_some() {
        let payload = from_json::<ValidSignaturesPayload>(payload.unwrap())?;
        validate_multi_payload(deps.storage, &payload)?;
        Some(payload)
    } else {
        None
    };

    let are_valid: Vec<bool> = signatures
        .into_iter()
        .enumerate()
        .map(|(index, signature)| {
            let data = data[index].clone();
            let credential_res = get_verifying_indexed_credential(
                deps,
                data.into(),
                signature.into(),
                index,
                &payload,
            );
            if credential_res.is_err() {
                return false;
            }
            credential_res.unwrap()
            .verified_cosmwasm(deps.api, &env, &None).is_ok()
        })
        .collect();

    Ok(ValidSignaturesResponse { are_valid })
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
        token_info: TOKEN_INFO.load(deps.storage)?,
        status: STATUS.load(deps.storage)?,
    })
}
