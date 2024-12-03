use cosmwasm_std::{
    ensure, Api, CosmosMsg, Deps, StdError, StdResult, Storage, WasmMsg
};
use cw_ownable::is_owner;
use cw_tba::ExecuteAccountMsg;
use saa::CredentialData;


use crate::{
    error::ContractError,
    state::{
        REGISTRY_ADDRESS, STATUS, WITH_CALLER
    },
};



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
    let res = REGISTRY_ADDRESS.load(store).map(|a| a == addr);
    if res.is_ok() && res.unwrap() {
        Ok(())
    } else {
        Err(ContractError::NotRegistry{})
    }
}


pub fn assert_owner_derivable(
    api: &dyn Api,
    storage: &mut dyn Storage,
    data: &CredentialData,
    owner: Option<String>
) -> Result<(), ContractError> {
    let owner = owner.unwrap_or(
        cw_ownable::get_ownership(storage)?.owner.unwrap().to_string()
    );

    ensure!(data
        .credentials
        .iter()
        .any(|c| {
            if !c.is_cosmos_derivable() {
                return false;
            }
            let addr = c.cosmos_address(api);
            if addr.is_err() {
                return false;
            }
            addr.unwrap().to_string() == owner
        }), 
        ContractError::NoOwnerCred {}
    );

    Ok(())
}






pub fn assert_valid_signed_action(action: &ExecuteAccountMsg) -> Result<(), ContractError> {
    match action {
        ExecuteAccountMsg::Extension { .. } => Err(ContractError::BadSignedAction(String::from(
            "Nested 'Extension' is not supported",
        ))),
        ExecuteAccountMsg::ReceiveNft { .. } => Err(ContractError::BadSignedAction(
            String::from("ReceiveNft can only be called by an NFT contract"),
        )),
        ExecuteAccountMsg::UpdateAccountData { .. } => Err(ContractError::NotRegistry {}),
        ExecuteAccountMsg::UpdateOwnership { .. } => Err(ContractError::NotRegistry {}),
        ExecuteAccountMsg::Purge {} => Err(ContractError::NotRegistry {}),
        _ => Ok(()),
    }
}





pub fn assert_caller(
    deps: Deps,
    sender: &str,
) -> Result<(), ContractError> {
    ensure!(
        WITH_CALLER.load(deps.storage)?,
        ContractError::NoDirectCall {}
    );
    ensure!(
        is_owner(deps.storage, &deps.api.addr_validate(sender)?)?,
        ContractError::Unauthorized("Only the owner can call directly".to_string())
    );
    Ok(())
}




pub fn assert_ok_wasm_msg(msg: &WasmMsg) -> StdResult<()> {
    match msg {
        _ => Err(StdError::generic_err("Not Supported")),
    }
}


pub fn assert_ok_cosmos_msg(msg: &CosmosMsg) -> StdResult<()> {
    match msg {
        CosmosMsg::Wasm(msg) => assert_ok_wasm_msg(msg),
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