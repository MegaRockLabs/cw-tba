use cosmwasm_std::{Addr, StdResult, Binary, StdError, WasmMsg, Storage, QuerierWrapper, CanonicalAddr, CosmosMsg};

use k256::sha2::{Sha256, Digest};
use bech32::{ToBase32, Variant};
use ripemd::Ripemd160;

use crate::{error::ContractError, state::{STATUS, REGISTRY_ADDRESS}};

pub const HRP: &str = "stars";


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



pub fn generate_amino_transaction_string(signer: &str, data: &str) -> String {
    format!(
        "{{\"account_number\":\"0\",\"chain_id\":\"\",\"fee\":{{\"amount\":[],\"gas\":\"0\"}},\"memo\":\"\",\"msgs\":[{{\"type\":\"sign/MsgSignData\",\"value\":{{\"data\":\"{}\",\"signer\":\"{}\"}}}}],\"sequence\":\"0\"}}", 
        data, signer
    )
}


pub fn pubkey_to_account(pubkey: &Binary) -> String {
    let base32_addr = pubkey_to_canonical(pubkey).0.as_slice().to_base32();
    let account: String = bech32::encode(HRP, base32_addr, Variant::Bech32).unwrap();
    account
}


fn pubkey_to_canonical(pubkey: &Binary) -> CanonicalAddr {
    let mut hasher = Ripemd160::new();
    hasher.update(sha_256(&pubkey.0));
    CanonicalAddr(Binary(hasher.finalize().to_vec()))
}


fn sha_256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();

    let mut result = [0u8; 32];
    result.copy_from_slice(hash.as_slice());
    result
}