use crate::{
    error::ContractError,
    state::{REGISTRY_ADDRESS, STATUS},
};
use cosmwasm_std::{ensure, Addr, Api, Binary, CosmosMsg, QuerierWrapper, StdError, StdResult, Storage, WasmMsg};
use saa_wasm::saa_types::{Credential, CredentialData};


pub fn assert_status(store: &dyn Storage) -> StdResult<bool> {
    let status = STATUS.load(store)?;
    Ok(!status.frozen)
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
    cw83::Cw83RegistryBase(addr).supports_interface(querier)
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


pub fn extract_pubkey(
    api: &dyn Api,
    data: CredentialData,
    owner: &Addr,
) -> Result<Binary, ContractError> {
    let credentials = data.credentials;

    ensure!(credentials.len() == 1, ContractError::PubkeyOnly {});
    let cred = credentials.first().unwrap();
    ensure!(cred.is_cosmos_derivable(), ContractError::PubkeyOnly {});
    
    let pubkey = if let Credential::CosmosArbitrary(cosmos_cred) = cred {
        let addr = cred.cosmos_address(api).map_err(|_| ContractError::PubkeyOnly {})?;
        ensure!(addr == owner, ContractError::Unauthorized {});
        cosmos_cred.pubkey.clone()
    } else {
        return Err(ContractError::PubkeyOnly {});
    };

    Ok(pubkey)
}



pub fn change_cosmos_msg(msg: cw_tba::CosmosMsg) -> Result<cosmwasm_std::CosmosMsg, ContractError>{
    Ok(match msg {
        CosmosMsg::Bank(msg) => cosmwasm_std::CosmosMsg::Bank(msg),
        CosmosMsg::Staking(msg) => cosmwasm_std::CosmosMsg::Staking(msg),
        CosmosMsg::Distribution(msg) => cosmwasm_std::CosmosMsg::Distribution(msg),
        CosmosMsg::Stargate { type_url, value } => cosmwasm_std::CosmosMsg::Stargate { type_url, value },
        CosmosMsg::Ibc(msg) => cosmwasm_std::CosmosMsg::Ibc(msg),
        CosmosMsg::Wasm(msg) => cosmwasm_std::CosmosMsg::Wasm(msg),
        CosmosMsg::Gov(gov_msg) => cosmwasm_std::CosmosMsg::Gov(gov_msg),
        CosmosMsg::Custom(_) => {
            return  Err(ContractError::Generic(String::from("Nested signing notsupported")))?;
        }
        _ => panic!("Unsupported message type"),
    })
}