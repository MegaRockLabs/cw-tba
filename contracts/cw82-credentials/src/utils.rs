use cosmwasm_std::{
    from_json, to_json_binary, to_json_string, Addr, Api, CosmosMsg, Deps, DepsMut, Empty, Env, QuerierWrapper, StdError, StdResult, Storage, WasmMsg
};
use cw_ownable::is_owner;
use cw_tba::ExecuteAccountMsg;
use saa::{
    cosmos_utils::{pubkey_to_account, pubkey_to_canonical},
    ensure,
    Binary, Caller, CosmosArbitrary, Credential, CredentialData, CredentialId, Ed25519,
    EvmCredential, Secp256k1, Verifiable,
};
use std::collections::HashSet;

use crate::{
    error::ContractError,
    msg::{
        AccountActionDataToSign, AuthPayload, CosmosMsgDataToSign, SignedAccountActions,
        SignedCosmosMsgs, ValidSignaturesPayload,
    },
    state::{
        CredentialInfo, CREDENTIALS, NONCES, REGISTRY_ADDRESS, STATUS,
        VERIFYING_CRED_ID, WITH_CALLER,
    },
};

const ONLY_ONE_ERR: &str = "Only one of the 'address' or 'hrp' can be provided";

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

pub fn assert_ok_wasm_msg(msg: &WasmMsg) -> StdResult<()> {
    let bad_wasm_error = StdError::GenericErr {
        msg: "Not Supported".into(),
    };
    match msg {
        WasmMsg::Execute { .. } => Err(bad_wasm_error),
        _ => Err(bad_wasm_error),
    }
}

pub fn assert_ok_cosmos_custom_msg(msg: &CosmosMsg<SignedCosmosMsgs>) -> StdResult<()> {
    let bad_msg_error = StdError::GenericErr {
        msg: "Not Supported".into(),
    };
    match msg {
        CosmosMsg::Wasm(msg) => assert_ok_wasm_msg(msg),
        CosmosMsg::Stargate { .. } => Err(bad_msg_error),
        _ => Ok(()),
    }
}

pub fn is_ok_cosmos_custom_msg(msg: &CosmosMsg<SignedCosmosMsgs>) -> bool {
    assert_ok_cosmos_custom_msg(msg).is_ok()
}

pub fn assert_ok_cosmos_msg(msg: &CosmosMsg<Empty>) -> StdResult<()> {
    let bad_msg_error = StdError::GenericErr {
        msg: "Not Supported".into(),
    };
    match msg {
        CosmosMsg::Wasm(msg) => assert_ok_wasm_msg(msg),
        CosmosMsg::Stargate { .. } => Err(bad_msg_error),
        _ => Ok(()),
    }
}

pub fn is_ok_cosmos_msg(msg: &CosmosMsg<Empty>) -> bool {
    assert_ok_cosmos_msg(msg).is_ok()
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
            message: from_json(&message)?,
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
        _ => {
            return Err(StdError::generic_err("Unsupported credential type"));
        }
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

fn get_digest_credential(
    deps: Deps,
    message: impl Into<Binary>,
    signature: impl Into<Binary>,
    payload: &Option<AuthPayload>,
) -> Result<Credential, ContractError> {

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

fn get_cosmos_msg_credential(
    deps: Deps,
    data: &CosmosMsgDataToSign,
    signature: Binary,
    payload: &Option<AuthPayload>,
) -> Result<Credential, ContractError> {
    let data = to_json_binary(&to_json_string(&data)?)?;
    get_digest_credential(deps, data, signature, &payload)
}

pub fn get_account_action_credential(
    deps: Deps,
    data: &AccountActionDataToSign,
    signature: Binary,
    payload: &Option<AuthPayload>,
) -> Result<Credential, ContractError> {
    let data = to_json_binary(&to_json_string(&data)?)?;
    get_digest_credential(deps, data, signature, &payload)
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

pub fn assert_owner_derivable(deps: Deps, data: &CredentialData) -> Result<(), ContractError> {
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


fn assert_valid_signed_action(action: &ExecuteAccountMsg) -> Result<(), ContractError> {
    match action {
        ExecuteAccountMsg::Execute { .. } => Err(ContractError::BadSignedAction(String::from(
            "'Execute' must be called directly",
        ))),
        ExecuteAccountMsg::UpdateAccountData { .. } => Err(ContractError::BadSignedAction(
            String::from("'UpdateAccountData' must be called directly"),
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

pub fn assert_signed_actions(
    deps: Deps,
    env: &Env,
    signed: &SignedAccountActions,
) -> Result<(), ContractError> {
    ensure!(!NONCES.has(deps.storage, signed.data.nonce.u128()), ContractError::NonceExists {});

    let credential = get_account_action_credential(
        deps,
        &signed.data,
        signed.signature.clone().into(),
        &signed.payload,
    )?;

    credential.verified_cosmwasm(deps.api, &env, &None)?;
    
    signed
        .data
        .actions
        .iter()
        .map(|action| assert_valid_signed_action(action))
        .collect::<Result<Vec<()>, ContractError>>()?;

    Ok(())
}

pub fn assert_signed_msg(
    deps: Deps,
    env: &Env,
    _sender: &str,
    msg: &SignedCosmosMsgs,
) -> Result<(), ContractError> {
    let credential =
        get_cosmos_msg_credential(deps, &msg.data, msg.signature.clone().into(), &msg.payload)?;

    ensure!(!NONCES.has(deps.storage, msg.data.nonce.u128()), ContractError::NonceExists {});
    credential.verified_cosmwasm(deps.api, env, &None)?;

    Ok(())
}

pub fn assert_simple_msg(
    deps: Deps,
    _env: &Env,
    sender: &str,
    _msg: &CosmosMsg<SignedCosmosMsgs>,
) -> Result<(), ContractError> {
    ensure!(
        WITH_CALLER.load(deps.storage)?,
        StdError::generic_err("Calling directly is not allowed. Message must be signed")
    );

    ensure!(
        is_owner(deps.storage, &deps.api.addr_validate(sender)?)?,
        ContractError::Unauthorized {}
    );

    if let CosmosMsg::Custom(_) = _msg {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}

pub fn assert_cosmos_msg(
    deps: Deps,
    env: &Env,
    sender: &str,
    msg: &CosmosMsg<SignedCosmosMsgs>,
) -> Result<(), ContractError> {
    match msg {
        CosmosMsg::Custom(signed) => assert_signed_msg(deps, env, sender, signed),
        _ => assert_simple_msg(deps, env, sender, msg),
    }
}

pub fn checked_execute_msg(
    deps: Deps,
    env: &Env,
    sender: &str,
    msg: &CosmosMsg<SignedCosmosMsgs>,
) -> Result<Vec<CosmosMsg>, ContractError> {
    match msg.clone() {
        CosmosMsg::Custom(signed) => {
            assert_signed_msg(deps, env, sender, &signed)?;

            signed
                .data
                .messages
                .iter()
                .map(|msg| assert_ok_cosmos_msg(msg))
                .collect::<StdResult<Vec<()>>>()?;

            Ok(signed.data.messages)
        }

        msg => {
            assert_simple_msg(deps, env, sender, &msg)?;

            let msg: CosmosMsg = match msg {
                CosmosMsg::Bank(b) => b.into(),
                CosmosMsg::Staking(s) => s.into(),
                CosmosMsg::Distribution(d) => d.into(),
                CosmosMsg::Gov(g) => g.into(),
                CosmosMsg::Ibc(ibc) => ibc.into(),
                CosmosMsg::Wasm(w) => w.into(),
                CosmosMsg::Stargate { type_url, value } => CosmosMsg::Stargate { type_url, value },
                CosmosMsg::Custom(_) => unreachable!(),
                _ => return Err(ContractError::NotSupported {}),
            };
            assert_ok_cosmos_msg(&msg)?;
            Ok(vec![msg])
        }
    }
}

pub fn checked_execute_msgs(
    deps: DepsMut,
    env: &Env,
    sender: &str,
    msgs: &Vec<CosmosMsg<SignedCosmosMsgs>>,
) -> Result<Vec<CosmosMsg>, ContractError> {
    let mut checked: Vec<CosmosMsg> = Vec::with_capacity(msgs.len() + 2);

    for msg in msgs.iter() {
        let extracted = checked_execute_msg(deps.as_ref(), env, sender, msg)?;

        if extracted.len() > 1 {
            if let CosmosMsg::Custom(signed) = &msg {
                NONCES.save(deps.storage, signed.data.nonce.u128(), &true)?;
            }
        }

        checked.extend(extracted);
    }

    Ok(checked)
}
