use cosmwasm_std::{ensure, ensure_eq, CosmosMsg, StdError, StdResult, Storage};
use saa_wasm::saa_types::CredentialRecord;

use crate::{
    error::ContractError,
    state::{REGISTRY_ADDRESS, STATUS},
};

pub fn assert_status(store: &dyn Storage) -> StdResult<()> {
    let status = STATUS.load(store)?;
    ensure!(
        !status.frozen,
        StdError::generic_err(ContractError::Frozen {}.to_string())
    );
    Ok(())
}


pub fn assert_registry(store: &dyn Storage, addr: &str) -> Result<(), ContractError> {
    let res = REGISTRY_ADDRESS.load(store)?;
    ensure_eq!(res.as_str(), addr, ContractError::NotRegistry {});
    Ok(())
}

pub fn assert_owner_derivable(
    creds: &Vec<CredentialRecord>,
    owner: &str,
) -> Result<(), ContractError> {
    let found = creds.iter().any(|(_, i)| {
        i.address
            .as_ref()
            .map(|a| a.to_string() == owner)
            .unwrap_or_default()
    });
    ensure!(found, ContractError::NoOwnerCred {});
    Ok(())
}

pub fn assert_ok_cosmos_msg(msg: &CosmosMsg) -> StdResult<()> {
    match msg {
        CosmosMsg::Wasm(_) => Err(StdError::generic_err("Not Supported")),
        CosmosMsg::Stargate { type_url, .. } => {
            if type_url.to_lowercase().contains("authz") {
                Err(StdError::generic_err("Not Supported"))
            } else {
                Ok(())
            }
        }
        _ => Ok(()),
    }
}
