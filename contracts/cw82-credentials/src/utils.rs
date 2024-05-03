use cosmwasm_std::{from_json, to_json_binary, Addr, Binary, CosmosMsg, Deps, QuerierWrapper, StdError, StdResult, Storage, WasmMsg};
use saa::{ensure, CosmosArbitrary, Credential, CredentialId, Ed25519, EvmCredential, Secp256k1};
use std::collections::HashSet;


use crate::{error::ContractError, msg::{ValidSignaturePayload, ValidSignaturesPayload}, state::{CredentialInfo, CREDENTIALS, REGISTRY_ADDRESS, STATUS, VERIFYING_CRED_ID}};


const ONLY_ONE_ERR : &str = "Only one of the 'address' or 'hrp' can be provided";


pub fn assert_status(
    store: &dyn Storage
) -> StdResult<bool>{
    let status = STATUS.load(store)?;
    Ok(!status.frozen)
}   

pub fn status_ok(
    store: &dyn Storage
) -> bool {
    assert_status(store).is_ok()
}


pub fn assert_ok_wasm_msg(
    msg: &WasmMsg
) -> StdResult<()> {
    let bad_wasm_error  = StdError::GenericErr { msg: "Not Supported".into() };
    match msg {
        // todo: add whitelististed messages
        WasmMsg::Execute { .. } => Err(bad_wasm_error),
        _ => Err(bad_wasm_error)
    }
}


pub fn assert_ok_cosmos_msg(
    msg: &CosmosMsg
) -> StdResult<()> {
    let bad_msg_error = StdError::GenericErr { msg: "Not Supported".into() };
    match msg {
        CosmosMsg::Wasm(msg) => assert_ok_wasm_msg(msg),
        CosmosMsg::Stargate { .. } => Err(bad_msg_error),
        _ => Ok(())
    }
}

pub fn is_ok_cosmos_msg(
    msg: &CosmosMsg
) -> bool {
    assert_ok_cosmos_msg(msg).is_ok()
}


pub fn query_if_registry(
    querier: &QuerierWrapper,
    addr: Addr
) -> StdResult<bool> {
    cw83::Cw83RegistryBase(addr).supports_interface(querier)
}



pub fn assert_registry(
    store: &dyn Storage,
    addr: &Addr
) -> Result<(), ContractError> {
    if is_registry(store, addr)? {
        Ok(())
    } else {
        Err(ContractError::Unauthorized {})
    }
}


pub fn is_registry(
    store: &dyn Storage,
    addr: &Addr
) -> StdResult<bool> {
    REGISTRY_ADDRESS.load(store).map(|a| a == addr.to_string())
}



fn validate_payload(
    storage: &dyn Storage,
    payload: &ValidSignaturePayload,
) -> StdResult<()> {
    
    if payload.hrp.is_some() {
        ensure!(payload.address.is_none(), StdError::generic_err(ONLY_ONE_ERR));
    }
    
    if payload.address.is_some() {
        ensure!(payload.hrp.is_none(), StdError::generic_err(ONLY_ONE_ERR));
    }

    if payload.credential_id.is_some() {

        let info_res = CREDENTIALS.load(
            storage, 
            payload.credential_id.clone().unwrap()
        );

        ensure!(
            info_res.is_ok(),
            StdError::generic_err("Credential not found")
        );


        if payload.hrp.is_some() {
            ensure!(
                info_res.unwrap().name == "cosmos-arbitrary",
                StdError::generic_err("hrp can only be used with cosmos-arbitrary")
            );
        }
    }

    Ok(())
}


pub fn validate_multi_payload(
    storage:  &dyn Storage,
    payload: &ValidSignaturesPayload
) -> StdResult<()> {

    match payload {
        ValidSignaturesPayload::Generic(p) => {
            validate_payload(storage, p)?;
        },
        ValidSignaturesPayload::Multiple(p) => {
            let count = p.len();
            ensure!(count < 255, StdError::generic_err("Too many payloads"));
            let mut indeces : HashSet<u8> = HashSet::with_capacity(count);
            p
                .iter()
                .map(|p| {
                    if p.is_none() {
                        return Ok(());
                    }
                    let p = p.clone().unwrap();
                    ensure!(p.index < 255 && p.index < count as u8, StdError::generic_err("Invalid index"));
                    let inserted = indeces.insert(p.index);
                    ensure!(!inserted, StdError::generic_err("Duplicate index"));
                    validate_payload(storage, &p.payload)
                })
                .collect::<StdResult<Vec<()>>>()?;

            // at least one must be specified
            ensure!(indeces.len() > 0, StdError::generic_err("No valid payloads"));
        }
    }
    Ok(())
}



fn get_verifying_credential_tuple(
    storage  : &dyn Storage,
    payload  : &Option<ValidSignaturePayload>,
    validate : bool
) -> StdResult<(CredentialId, CredentialInfo)> {
    let id = match payload {
        Some(payload) => {
            if validate {
                validate_payload(storage, &payload)?;
            }
            payload.credential_id.clone().unwrap_or(
                VERIFYING_CRED_ID.load(storage)?
            )
        },
        None => {
            VERIFYING_CRED_ID.load(storage)?
        }
    };
    let info = CREDENTIALS.load(storage, id.clone())?;
    Ok((id, info))
}



fn get_credential_from_args(
    id          : CredentialId,
    info        : CredentialInfo,
    data        : Binary,
    signature   : Binary,
    payload     : &Option<ValidSignaturePayload>
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
                        Some(address) => {
                            to_json_binary(address)?.0
                        },
                        None => id
                    }
                },
                None => id
            };
            Credential::Evm(EvmCredential {
                message: data.into(),
                signature: signature.into(),
                signer,
            })
        }
        "cosmos-arbitrary" => {
            Credential::CosmosArbitrary(CosmosArbitrary {
                pubkey: id,
                message: data.into(),
                signature: signature.into(),
                hrp: payload.clone().map(|p| p.hrp).unwrap_or(info.hrp)
            })
        }
        "ed25519" => {
            Credential::Ed25519(Ed25519 {
                pubkey: id,
                message: data.into(),
                signature: signature.into()
            })
        },
        "secp256k1" => {
            Credential::Secp256k1(Secp256k1 {
                pubkey: id,
                message: data.into(),
                signature: signature.into()
            })
        },
        _ => {
            return Err(StdError::generic_err("Unsupported credential type"));
        }
    };

    Ok(credential)
}



pub fn get_verifying_credential(
    deps        : Deps,
    data        : Binary,
    signature   : Binary,
    payload     : &Option<Binary>,
) -> StdResult<Credential> {

    let payload = match payload {
        Some(payload) => Some(from_json(payload)?),
        None => None
    };

    let (id, info) = get_verifying_credential_tuple(deps.storage, &payload, true)?;

    get_credential_from_args(
        id, 
        info, 
        data, 
        signature, 
        &payload
    )
}


pub fn get_verifying_indexed_credential(
    deps        : Deps,
    data        : Binary,
    signature   : Binary,
    index       : usize,
    payload     : &Option<ValidSignaturesPayload>,
) -> StdResult<Credential> {

    let payload = match payload {
        Some(payload) => {
            let payload = match payload {
                ValidSignaturesPayload::Generic(p) => {
                    Some(p.clone())
                },
                ValidSignaturesPayload::Multiple(p) => {
                    p.get(index).map(|op| 
                        op.clone().map(|ip| ip.payload)
                    ).flatten()
                }
            };
            payload
        }
        None => None
    };

    let (id, info) = get_verifying_credential_tuple(deps.storage, &payload, true)?;

    get_credential_from_args(
        id, 
        info,
        data, 
        signature, 
        &payload
    )
}
