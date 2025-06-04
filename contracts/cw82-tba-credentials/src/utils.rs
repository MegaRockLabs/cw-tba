use cosmwasm_std::{ensure, ensure_eq, CosmosMsg, StdError, StdResult, Storage};
use saa_wasm::saa_types::{CredentialRecord, VerifiedData};


use crate::{error::ContractError, state::{REGISTRY_ADDRESS, STATUS}};


pub fn assert_status(store: &dyn Storage) -> StdResult<()> {
    let status = STATUS.load(store)?;
    ensure!(!status.frozen, StdError::generic_err(ContractError::Frozen {}.to_string()));
    Ok(())
}

pub fn status_ok(store: &dyn Storage) -> bool {
    assert_status(store).is_ok()
}

#[cfg(target_arch = "wasm32")]
pub fn query_if_registry(querier: &cosmwasm_std::QuerierWrapper, addr: cosmwasm_std::Addr) -> StdResult<bool> {
    let key = cosmwasm_std::storage_keys::namespace_with_key(
        &[cw22::INTERFACE_NAMESPACE.as_bytes()], 
        "crates:cw83".as_bytes()
    );
    let raw_query = cosmwasm_std::WasmQuery::Raw { 
        contract_addr: addr.to_string(),
        key: key.into()
    };
    let version : Option<String> = querier.query(&cosmwasm_std::QueryRequest::Wasm(raw_query))?;
    Ok(version.is_some())
}


pub fn assert_registry(store: &dyn Storage, addr: &str) -> Result<(), ContractError> {
    let res = REGISTRY_ADDRESS.load(store)?;
    ensure_eq!(res.as_str(), addr, ContractError::NotRegistry {});
    Ok(())
}


pub fn assert_data_owner_derivable(
    data: &VerifiedData,
    owner: String,
) -> Result<(), ContractError> {
    ensure!(
        data.addresses.iter().any(|addr| *addr == owner),
        ContractError::NoOwnerCred {}
    );
    Ok(())
}



pub fn assert_owner_derivable(
    records: Vec<CredentialRecord>,
    owner: String,
) -> Result<(), ContractError> {
    ensure!(
        records.iter().any(|record| {
            if let Some(addr) = record.1.address.clone() {
                addr == owner
            } else {
                false
            }
        }),
        ContractError::NoOwnerCred {}
    );
    Ok(())
}






pub fn assert_ok_cosmos_msg(msg: &CosmosMsg) -> StdResult<()> {
    match msg {
        CosmosMsg::Wasm(_) => Err(StdError::generic_err("Not Supported")),
        CosmosMsg::Stargate { type_url, .. }  => {
            if type_url.to_lowercase().contains("authz") {
                Err(StdError::generic_err("Not Supported"))
            } else {
                Ok(())
            }
        },
        _ => Ok(()),
    }
}