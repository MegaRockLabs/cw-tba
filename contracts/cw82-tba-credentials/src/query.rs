use cosmwasm_std::{from_json, CosmosMsg, Deps, Env, Order, StdError, StdResult, Binary};
use cw82::{CanExecuteResponse, ValidSignatureResponse};
use cw_tba::{AssetsResponse, FullInfoResponse, TokenInfo};
use cw_auths::{saa_types::msgs::{AuthPayload, SignedDataMsg}, verify_native, verify_signed};

use crate::{
    state::{KNOWN_TOKENS, REGISTRY_ADDRESS, STATUS, TOKEN_INFO},
    utils::
        status_ok
    ,
};

const DEFAULT_BATCH_SIZE: u32 = 100;


pub fn can_execute(
    deps: Deps,
    env: Env,
    sender: String,
    msg: CosmosMsg<SignedDataMsg>,
) -> StdResult<CanExecuteResponse> {
    if !status_ok(deps.storage) {
        return Ok(CanExecuteResponse { can_execute: false });
    };

    let can_execute = match msg {
        CosmosMsg::Custom(
            signed
        ) => {
            verify_signed(deps.api, deps.storage, &env, signed).is_ok()
        },
        _ => verify_native( deps.storage, sender).is_ok()
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

        let payload = payload
            .map(|p| from_json::<AuthPayload>(p)).transpose()?;

        verify_signed(
            deps.api,
            deps.storage, 
            &env,
            SignedDataMsg { data, signature, payload },
        )
        .is_ok()

    } else {
        false
    };

    Ok(ValidSignatureResponse { is_valid })
}

/* pub fn valid_signatures(
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

    let payload = payload
            .map(|p| from_json::<AuthPayload>(p)).transpose()?;
    

    let are_valid: Vec<bool> = signatures
        .into_iter()
        .enumerate()
        .map(|(index, signature)| {
            let data = data[index].clone();
            cw_auths::verify_signed(
                deps.api,
                deps.storage, 
                &env,
                SignedDataMsg { 
                    data, 
                    signature, 
                    payload: payload.clone()
                },
            )
            .is_ok()
         
        })
        .collect();

    Ok(ValidSignaturesResponse { are_valid })
} */


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
    let credentials = cw_auths::get_stored_credentials(deps.storage)
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
