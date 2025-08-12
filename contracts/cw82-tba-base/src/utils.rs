use crate::{
    error::ContractError,
    state::{REGISTRY_ADDRESS, STATUS},
};
use cosmwasm_std::{
    ensure, Addr, Binary, CosmosMsg, QuerierWrapper, StdError, StdResult, Storage, WasmMsg,
};
use saa_wasm::saa_types::{CredentialAddress, VerifiedData};

pub fn assert_status(store: &dyn Storage) -> StdResult<()> {
    let status = STATUS.load(store)?;
    if status.frozen {
        return Err(StdError::generic_err("Account is frozen"));
    }
    Ok(())
}

pub fn status_ok(store: &dyn Storage) -> bool {
    assert_status(store).is_ok()
}

pub fn assert_ok_wasm_msg(msg: &WasmMsg) -> StdResult<()> {
    let bad_wasm_error = StdError::generic_err("Not Supported");
    match msg {
        // todo: add whitelististed messages
        WasmMsg::Execute { .. } => Err(bad_wasm_error),
        _ => Err(bad_wasm_error),
    }
}

pub fn assert_ok_cosmos_msg(msg: &CosmosMsg) -> StdResult<()> {
    let bad_msg_error = StdError::generic_err("Not Supported");
    match msg {
        CosmosMsg::Wasm(msg) => assert_ok_wasm_msg(msg),
        CosmosMsg::Stargate { .. } => Err(bad_msg_error),
        _ => Ok(()),
    }
}

pub fn is_ok_cosmos_msg(msg: &CosmosMsg) -> bool {
    assert_ok_cosmos_msg(msg).is_ok()
}

pub fn query_if_registry(querier: &QuerierWrapper, addr: Addr) -> StdResult<bool> {
    Ok(cw22::query_supported_interface_version(querier, addr.as_str(), "crates:cw83")?.is_some())
}

pub fn assert_registry(store: &dyn Storage, addr: &Addr) -> Result<(), ContractError> {
    if is_registry(store, addr)? {
        Ok(())
    } else {
        Err(ContractError::Unauthorized {})
    }
}

pub fn is_registry(store: &dyn Storage, addr: &Addr) -> StdResult<bool> {
    REGISTRY_ADDRESS.load(store).map(|a| a == addr.to_string())
}

pub fn extract_pubkey(data: VerifiedData, owner: &Addr) -> Result<Binary, ContractError> {
    let credentials = data.credentials;

    ensure!(credentials.len() == 1, ContractError::PubkeyOnly {});
    let (id, info) = credentials.first().unwrap();

    if let Some(CredentialAddress::Bech32(a)) = info.address.as_ref() {
        ensure!(a == owner, ContractError::Unauthorized {});
        return Ok(Binary::from_base64(&id)?);
    } else {
        return Err(ContractError::NotSupported {});
    }
}
