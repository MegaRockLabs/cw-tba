use cosmwasm_std::{ensure, from_json, CosmosMsg, Deps, Env, Order, StdError, StdResult, Binary};
use cw82::{CanExecuteResponse, ValidSignatureResponse, ValidSignaturesResponse};
use cw_tba::{AssetsResponse, ExecuteAccountMsg, TokenInfo};
use saa::{
    load_credential, 
    messages::{AccountCredentials, AuthPayload, CredentialFullInfo, SignedData}, 
    storage::CREDENTIAL_INFOS, CredentialName, Verifiable
};

use crate::{
    msg::FullInfoResponse,
    state::{KNOWN_TOKENS, REGISTRY_ADDRESS, STATUS, TOKEN_INFO, VERIFYING_CRED_ID, WITH_CALLER},
    utils::{
        assert_caller, status_ok
    },
};

const DEFAULT_BATCH_SIZE: u32 = 100;


pub fn can_execute(
    deps: Deps,
    env: Env,
    sender: String,
    msg: CosmosMsg<SignedData<ExecuteAccountMsg>>,
) -> StdResult<CanExecuteResponse> {
    if !status_ok(deps.storage) {
        return Ok(CanExecuteResponse { can_execute: false });
    };

    let can_execute = match msg {
        CosmosMsg::Custom(
            signed
        ) => {
            signed.validate_cosmwasm(deps.storage, &env).is_ok()
        },
        _ => assert_caller(deps,  &sender).is_ok(),
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

        let credential = load_credential(
                deps.storage, data.into(), signature.into(), payload
            )
            .map_err(|_| StdError::generic_err("Credential not found"))?;

        credential.assert_query_cosmwasm::<ExecuteAccountMsg>(deps.api, deps.storage, &env, &None).is_ok()
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

    let payload = payload
            .map(|p| from_json::<AuthPayload>(p)).transpose()?;

    

    let are_valid: Vec<bool> = signatures
        .into_iter()
        .enumerate()
        .map(|(index, signature)| {
            let data = data[index].clone();
            let credential = load_credential(
                        deps.storage, data.into(), signature.into(), payload.clone()
                    )
                    .map_err(|_| StdError::generic_err("Credential not found"));

            if credential.is_err() {
                return false;
            }

            credential.unwrap()
            .assert_query_cosmwasm::<ExecuteAccountMsg>(deps.api, deps.storage, &env, &None)
            .is_ok()
         
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



pub fn credentials(
    deps: Deps,
) -> StdResult<AccountCredentials> {
    let credentials = CREDENTIAL_INFOS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (id, info) = item?;
            
            let human_id = match info.name == CredentialName::Passkey {
                false => String::from_utf8(id.clone()).unwrap(),
                true => Binary(id.clone()).to_base64(),
            };
            Ok(CredentialFullInfo {
                id,
                human_id,
                name: info.name,
                hrp: info.hrp,
                extension: info.extension,
            })
        })
        .collect::<StdResult<Vec<CredentialFullInfo>>>()?;

    let verifying_id = VERIFYING_CRED_ID.load(deps.storage)?;

    Ok(AccountCredentials {
        credentials,
        native_caller: WITH_CALLER.load(deps.storage)?,
        verifying_human_id: Binary(verifying_id.clone()).to_base64(),
        verifying_id: verifying_id,
    })

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
    let credentials = credentials(deps)?;

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
