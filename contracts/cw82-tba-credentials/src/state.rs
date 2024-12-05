use crate::{error::ContractError, utils::assert_owner_derivable};
use cosmwasm_std::{Api, ContractInfo, Env, MessageInfo, Storage};
use cw_ownable::initialize_owner;
use cw_storage_plus::{Item, Map};
use cw_tba::{Status, TokenInfo};
use saa::CredentialData;



pub static REGISTRY_ADDRESS: Item<String> = Item::new("r");
pub static TOKEN_INFO: Item<TokenInfo> = Item::new("t");
pub static STATUS: Item<Status> = Item::new("s");
pub static MINT_CACHE: Item<String> = Item::new("m");
pub static WITH_CALLER: Item<bool> = Item::new("w");
pub static KNOWN_TOKENS: Map<(&str, &str), bool> = Map::new("k");


pub fn save_credentials(
    api: &dyn Api,
    storage: &mut dyn Storage,
    env: &Env,
    info: MessageInfo,
    data: CredentialData,
    owner: String,
) -> Result<(), ContractError> {
    // Saving a flag wether info.sender has "root" rights 
    let with_caller = data.with_caller.unwrap_or_default();
    WITH_CALLER.save(storage, &with_caller)?;
    
    // On account creatino the signed contract address that of the registry
    // instead of the account contract
    let registry_env = Env {
        contract: ContractInfo { address: info.sender.clone() },
        ..env.clone()
    };

    // verify all credentials and save them
    data.save_cosmwasm(api, storage, &registry_env, &info)?;

    // save the owner adderss to the storage
    initialize_owner(storage, api, Some(owner.as_str()))?;

    if with_caller {
        // ensure that the owner is a valid cosmos address
        api.addr_validate(&owner)?;
    } else {
        // ensure that at least one of the credentials can be derived into the owner address
        assert_owner_derivable(api, storage, &data, Some(owner))?;
    }
    
    Ok(())
}
