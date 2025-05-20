use cosmwasm_std::{ensure, ensure_eq, Api, CosmosMsg, StdError, StdResult, Storage};
use cw_auths::saa_types::{msgs::SignedDataMsg, CredentialData};


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
            if let Ok(addr) = addr {
                return addr.to_string() == owner;
            }
            false
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


pub fn change_cosmos_msg(msg: CosmosMsg<SignedDataMsg>) -> Result<CosmosMsg, ContractError>{
    Ok(match msg {
        CosmosMsg::Bank(msg) => CosmosMsg::Bank(msg),
        CosmosMsg::Staking(msg) => CosmosMsg::Staking(msg),
        CosmosMsg::Distribution(msg) => CosmosMsg::Distribution(msg),
        CosmosMsg::Stargate { type_url, value } => CosmosMsg::Stargate { type_url, value },
        CosmosMsg::Ibc(msg) => CosmosMsg::Ibc(msg),
        CosmosMsg::Wasm(msg) => CosmosMsg::Wasm(msg),
        CosmosMsg::Gov(gov_msg) => CosmosMsg::Gov(gov_msg),
        CosmosMsg::Custom(_) => {
                        return  Err(ContractError::Generic(String::from("Nested signing notsupported")))?;
            }
        _ => panic!("Unsupported message type"),
    })
}

/* pub fn chane_action(msg: ExecuteMsg) -> Result<ExecuteAccountMsg, ContractError> {
    Ok(match msg {
        ExecuteMsg::Execute { msgs } => ExecuteAccountMsg::Execute { msgs },
        ExecuteMsg::UpdateOwnership { new_owner, new_account_data } => {
            ExecuteAccountMsg::UpdateOwnership { new_owner, new_account_data }
        },
        ExecuteMsg::UpdateAccountData { op } => ExecuteAccountMsg::UpdateAccountData { op },
        ExecuteMsg::ReceiveNft(msg) => ExecuteAccountMsg::ReceiveNft(msg),
        ExecuteMsg::Purge {} => ExecuteAccountMsg::Purge {},
        _ => panic!("Unsupported message type"),
    })
}


 */






