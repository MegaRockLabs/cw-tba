use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    from_json, to_json_string, Addr, Api, CosmosMsg, Deps, Env, QuerierWrapper, StdError, StdResult, Storage, WasmMsg
};
use cw_ownable::is_owner;
use cw_tba::ExecuteAccountMsg;
use saa::{
    cosmos_utils::{pubkey_to_account, pubkey_to_canonical}, ensure, Binary, Caller, ClientData, CosmosArbitrary, Credential, CredentialData, CredentialId, Ed25519, EvmCredential, PasskeyCredential, Secp256k1, Verifiable
};
use std::{collections::HashSet, fmt::Display};

use crate::{
    error::ContractError,
    msg::{
        ActionDataToSign, AuthPayload, SignedActions,
        ValidSignaturesPayload,
    },
    state::{
        CredentialInfo, CREDENTIALS, NONCES, REGISTRY_ADDRESS, STATUS,
        VERIFYING_CRED_ID, WITH_CALLER,
    },
};

const ONLY_ONE_ERR: &str = "Only one of the 'address' or 'hrp' can be provided";


#[cw_serde]
struct PasskeyExtension {
    id: String,
    client_data: ClientData,
    user_handle: Option<String>,
}


pub fn assert_status(store: &dyn Storage) -> StdResult<bool> {
    let status = STATUS.load(store)?;
    ensure!(
        !status.frozen,
        StdError::GenericErr {
            msg: ContractError::Frozen {}.to_string()
        }
    );
    Ok(true)
}

pub fn status_ok(store: &dyn Storage) -> bool {
    assert_status(store).is_ok()
}



pub fn query_if_registry(querier: &QuerierWrapper, addr: Addr) -> StdResult<bool> {
    cw83::Cw83RegistryBase(addr).supports_interface(querier)
}

pub fn assert_registry(store: &dyn Storage, addr: &Addr) -> Result<(), ContractError> {
    if is_registry(store, addr) {
        Ok(())
    } else {
        Err(ContractError::Unauthorized {})
    }
}

pub fn is_registry(store: &dyn Storage, addr: &Addr) -> bool {
    let res = REGISTRY_ADDRESS.load(store).map(|a| a == addr.to_string());
    res.is_ok() && res.unwrap()
}

fn validate_payload(storage: &dyn Storage, payload: &AuthPayload) -> StdResult<()> {
    if payload.hrp.is_some() {
        ensure!(
            payload.address.is_none(),
            StdError::generic_err(ONLY_ONE_ERR)
        );
    }

    if payload.address.is_some() {
        ensure!(payload.hrp.is_none(), StdError::generic_err(ONLY_ONE_ERR));
    }

    if payload.credential_id.is_some() {
        let info_res = CREDENTIALS.load(storage, payload.credential_id.clone().unwrap());

        ensure!(
            info_res.is_ok(),
            StdError::generic_err("Credential not found")
        );

        if payload.hrp.is_some() {
            let name = info_res.unwrap().name;
            ensure!(
                name == "cosmos-arbitrary" || name == "secp256k1",
                StdError::generic_err(
                    "'hrp' can only be used with 'cosmos-arbitrary' or 'secp256k1'"
                )
            );
        }
    }

    Ok(())
}


pub fn validate_multi_payload(
    storage: &dyn Storage,
    payload: &ValidSignaturesPayload,
) -> StdResult<()> {
    match payload {
        ValidSignaturesPayload::Generic(p) => {
            validate_payload(storage, p)?;
        }
        ValidSignaturesPayload::Multiple(p) => {
            let count = p.len();
            ensure!(count < 255, StdError::generic_err("Too many payloads"));
            let mut indeces: HashSet<u8> = HashSet::with_capacity(count);
            p.iter()
                .map(|p| {
                    if p.is_none() {
                        return Ok(());
                    }
                    let p = p.clone().unwrap();
                    ensure!(
                        p.index < 255 && p.index < count as u8,
                        StdError::generic_err("Invalid index")
                    );
                    let inserted = indeces.insert(p.index);
                    ensure!(!inserted, StdError::generic_err("Duplicate index"));
                    validate_payload(storage, &p.payload)
                })
                .collect::<StdResult<Vec<()>>>()?;

            // at least one must be specified
            ensure!(
                indeces.len() > 0,
                StdError::generic_err("No valid payloads")
            );
        }
    }
    Ok(())
}

pub fn get_verifying_credential_tuple(
    storage: &dyn Storage,
    payload: &Option<AuthPayload>,
    validate: bool,
) -> StdResult<(CredentialId, CredentialInfo)> {
    let id = match payload {
        Some(payload) => {
            if validate {
                validate_payload(storage, &payload)?;
            }
            payload
                .credential_id
                .clone()
                .unwrap_or(VERIFYING_CRED_ID.load(storage)?)
        }
        None => VERIFYING_CRED_ID.load(storage)?,
    };
    let info = CREDENTIALS.load(storage, id.clone())?;
    Ok((id, info))
}


pub fn get_credential_from_args(
    id: CredentialId,
    info: CredentialInfo,
    message: Binary,
    signature: Binary,
    payload: &Option<AuthPayload>,
) -> StdResult<Credential> {
    let credential = match info.name.as_str() {
        "evm" => {
            let signer = match payload {
                Some(payload) => {
                    ensure!(
                        payload.hrp.is_none(),
                        StdError::generic_err("Cannot use 'hrp' with EVM credentials")
                    );
                    match payload.address.as_ref() {
                        Some(address) => address.clone(),
                        None => String::from_utf8(id)?,
                    }
                }
                None => String::from_utf8(id)?,
            };
            Credential::Evm(EvmCredential {
                message,
                signature,
                signer,
            })
        }
        "cosmos-arbitrary" => Credential::CosmosArbitrary(CosmosArbitrary {
            pubkey: Binary(id),
            message: message.to_string(),
            signature,
            hrp: payload.clone().map(|p| p.hrp).unwrap_or(info.hrp),
        }),
        "ed25519" => Credential::Ed25519(Ed25519 {
            pubkey: Binary(id),
            message,
            signature,
        }),
        "secp256k1" => Credential::Secp256k1(Secp256k1 {
            pubkey: Binary(id),
            message,
            signature,
            hrp: payload.clone().map(|p| p.hrp).unwrap_or(info.hrp),
        }),
        "caller" => Credential::Caller(Caller { id }),

        "passkey" => {
            ensure!(
                payload.is_some(),
                StdError::generic_err("Payload must be provided for 'passkey'")
            );
            let payload = payload.as_ref().unwrap();
            ensure!(
                payload.extension.is_some(),
                StdError::generic_err("Extension must be provided for 'passkey'")
            );

            let ext : PasskeyExtension = from_json(payload.extension.as_ref().unwrap())?;

            Credential::Passkey(PasskeyCredential {
                public_key: Some(id.into()),
                authenticator_data: message,
                client_data: ext.client_data,
                user_handle: ext.user_handle,
                id: ext.id,
                signature,
            })
        },
        _ => {
            return Err(StdError::generic_err("Unsupported credential type"));
        },
    };

    Ok(credential)
}


pub fn get_verifying_credential(
    deps: Deps,
    data: impl Into<Binary>,
    signature: impl Into<Binary>,
    payload: Option<impl Into<Binary>>,
) -> StdResult<Credential> {
    let payload = match payload {
        Some(payload) => Some(from_json::<AuthPayload>(&payload.into())?),
        None => None,
    };

    let (id, info) = get_verifying_credential_tuple(deps.storage, &payload, true)?;

    if info.name == "caller" {
        return Err(StdError::generic_err("Cannot verify payload with 'caller'"));
    }

    get_credential_from_args(id, info, data.into(), signature.into(), &payload)
}


pub fn get_verifying_indexed_credential(
    deps: Deps,
    data: Binary,
    signature: Binary,
    index: usize,
    payload: &Option<ValidSignaturesPayload>,
) -> StdResult<Credential> {
    let payload = match payload {
        Some(payload) => {
            let payload = match payload {
                ValidSignaturesPayload::Generic(p) => Some(p.clone()),
                ValidSignaturesPayload::Multiple(p) => p
                    .get(index)
                    .map(|op| op.clone().map(|ip| ip.payload))
                    .flatten(),
            };
            payload
        }
        None => None,
    };

    let (id, info) = get_verifying_credential_tuple(deps.storage, &payload, false)?;
    if info.name == "caller" {
        return Err(StdError::generic_err("Cannot verify payload with 'caller'"));
    }

    get_credential_from_args(id, info, data, signature, &payload)
}



fn derive_cosmos_address(
    api: &dyn Api,
    pubkey: &[u8],
    hrp: &Option<String>,
) -> Result<Addr, ContractError> {
    let address = if hrp.is_some() {
        api.addr_validate(&pubkey_to_account(pubkey, hrp.as_ref().unwrap())?)?
    } else {
        let canoncial = pubkey_to_canonical(pubkey);
        api.addr_humanize(&canoncial)?
    };
    Ok(address)
}




pub fn assert_owner_derivable<M : Display + Clone>(deps: Deps, data: &CredentialData<M>) -> Result<(), ContractError> {
    for cred in data.credentials.iter() {
        match cred {
            Credential::CosmosArbitrary(ca) => {
                if is_owner(
                    deps.storage,
                    &derive_cosmos_address(deps.api, &ca.pubkey, &ca.hrp)?,
                )? {
                    return Ok(());
                }
            }
            Credential::Secp256k1(s) => {
                if is_owner(
                    deps.storage,
                    &derive_cosmos_address(deps.api, &s.pubkey, &s.hrp)?,
                )? {
                    return Ok(());
                }
            }
            _ => {}
        }
    }

    Err(ContractError::NotDerivable {})
}




pub fn get_action_credential(
    deps: Deps,
    data: &ActionDataToSign,
    signature: Binary,
    payload: &Option<AuthPayload>,
) -> Result<Credential, ContractError> {
    let message = Binary(to_json_string(&data)?.as_bytes().to_vec());

    let (
        id, 
        info
    ) = get_verifying_credential_tuple(deps.storage, &payload, true)?;

    let cred = get_credential_from_args(
        id, 
        info, 
        message.into(), 
        signature.into(), 
        &payload
    )?;

    Ok(cred)
}




fn assert_valid_signed_action(action: &ExecuteAccountMsg) -> Result<(), ContractError> {
    match action {
        ExecuteAccountMsg::UpdateAccountData { .. } => Err(ContractError::BadSignedAction(
            String::from("'UpdateAccountData' must be called directly by the registry"),
        )),
        ExecuteAccountMsg::Extension { .. } => Err(ContractError::BadSignedAction(String::from(
            "Nested 'Extension' is not supported",
        ))),
        ExecuteAccountMsg::UpdateOwnership { .. } => Err(ContractError::Unauthorized {}),
        ExecuteAccountMsg::ReceiveNft { .. } => Err(ContractError::Unauthorized {}),
        ExecuteAccountMsg::Purge {} => Err(ContractError::Unauthorized {}),
        _ => Ok(()),
    }
}

pub fn assert_signed_msg(
    deps: Deps,
    env: &Env,
    signed: &SignedActions,
) -> Result<(), ContractError> {
    ensure!(!NONCES.has(deps.storage, signed.data.nonce.u128()), ContractError::NonceExists {});

    let credential = get_action_credential(
        deps,
        &signed.data,
        signed.signature.clone().into(),
        &signed.payload,
    )?;

    credential.verified_cosmwasm(deps.api, &env, &None)?;
    
    signed
        .data
        .messages
        .iter()
        .map(|action| assert_valid_signed_action(action))
        .collect::<Result<Vec<()>, ContractError>>()?;

    Ok(())
}



pub fn assert_caller(
    deps: Deps,
    sender: &str,
) -> Result<(), ContractError> {
    ensure!(
        WITH_CALLER.load(deps.storage)?,
        StdError::generic_err("Calling directly is not allowed. Message must be signed")
    );
    ensure!(
        is_owner(deps.storage, &deps.api.addr_validate(sender)?)?,
        ContractError::Unauthorized {}
    );
    Ok(())
}



pub fn uncustomised_cosmos_msg(msg: CosmosMsg<SignedActions>) -> CosmosMsg {
    match msg {
        CosmosMsg::Bank(b) => b.into(),
        CosmosMsg::Staking(s) => s.into(),
        CosmosMsg::Distribution(d) => d.into(),
        CosmosMsg::Gov(g) => g.into(),
        CosmosMsg::Ibc(ibc) => ibc.into(),
        CosmosMsg::Wasm(w) => w.into(),
        CosmosMsg::Stargate { type_url, value } => CosmosMsg::Stargate { type_url: type_url.clone(), value: value.clone() },
        _ => unreachable!(),
    }
}


pub fn assert_ok_wasm_msg(msg: &WasmMsg) -> StdResult<()> {
    match msg {
        _ => Err(StdError::generic_err("Not Supported")),
    }
}


pub fn assert_ok_cosmos_msg(msg: &CosmosMsg) -> StdResult<()> {
    match msg {
        CosmosMsg::Wasm(msg) => assert_ok_wasm_msg(msg),
        CosmosMsg::Stargate { .. } => Err(StdError::generic_err("Not Supported")),
        _ => Ok(()),
    }
}