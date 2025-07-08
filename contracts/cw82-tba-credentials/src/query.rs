use cosmwasm_std::{ensure, to_json_string, Binary, CosmosMsg, Deps, Env, Order, StdError, StdResult};
use cw84::{CanExecuteResponse, ValidSignatureResponse, ValidSignaturesResponse};
use cw_tba::{AssetsResponse, ExecuteAccountMsg, FullInfoResponse, TokenInfo};
use saa_wasm::{
    saa_types::{Credential}, verify_credential, verify_native
};

use crate::{
    state::{KNOWN_TOKENS, REGISTRY_ADDRESS, STATUS, TOKEN_INFO},
    utils::{assert_ok_cosmos_msg, assert_status},
};

const DEFAULT_BATCH_SIZE: u32 = 100;

pub fn can_execute(deps: Deps, sender: String, msg: CosmosMsg) -> StdResult<CanExecuteResponse> {
    Ok(CanExecuteResponse {
        can_execute: assert_status(deps.storage).is_ok()
            && verify_native(deps.storage, sender).is_ok()
            && assert_ok_cosmos_msg(&msg).is_ok(),
    })
}

pub fn can_execute_signed(
    deps: Deps,
    env: Env,
    cred: Credential,
    msg: Vec<ExecuteAccountMsg>,
) -> StdResult<CanExecuteResponse> {
    Ok(CanExecuteResponse {
        can_execute: assert_status(deps.storage).is_ok() && 
        verify_credential(deps, &env, cred, Some(vec![to_json_string(&msg)?])).is_ok(),
    })
}

#[allow(unused)]
pub fn valid_signature(
    deps: Deps,
    _env: Env,
    data: Binary,
    signature: Binary,
    payload: Option<Credential>,
) -> StdResult<ValidSignatureResponse> {
    Ok(ValidSignatureResponse {
        is_valid: payload.map(|c|c.verify(deps)).transpose().is_ok()
    })
}

pub fn valid_signatures(
    deps: Deps,
    _env: Env,
    data: Vec<Binary>,
    signatures: Vec<Binary>,
    payload: Option<Credential>,
) -> StdResult<ValidSignaturesResponse> {
    if assert_status(deps.storage).is_err() {
        return Ok(ValidSignaturesResponse {
            are_valid: vec![false, data.len() > 1],
        });
    };
    ensure!(
        data.len() == signatures.len(),
        StdError::generic_err("Data and signatures must be of equal length")
    );

    let cred = match payload {
        Some(c) => c,
        None => return Ok(ValidSignaturesResponse { are_valid: vec![false; data.len()] }),
    };

    return Ok(ValidSignaturesResponse {
        are_valid: vec![cred.verify(deps).is_ok(); data.len()],
    });

}


pub fn assets(
    deps: Deps,
    env: Env,
    skip: Option<u32>,
    limit: Option<u32>,
) -> StdResult<AssetsResponse> {
    Ok(AssetsResponse {
        balances: deps.querier.query_all_balances(env.contract.address)?,
        tokens: known_tokens(deps, skip, limit)?,
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
    let credentials = saa_wasm::get_stored_credentials(deps.storage)
        .map_err(|_| StdError::generic_err("Error getting credentials"))?;

    Ok(FullInfoResponse {
        balances,
        tokens,
        ownership,
        credentials,
        registry: REGISTRY_ADDRESS.load(deps.storage)?,
        token_info: TOKEN_INFO.load(deps.storage)?,
        status: STATUS.load(deps.storage)?,
    })
}
