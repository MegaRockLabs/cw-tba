use crate::{error::ContractError, utils::assert_data_owner_derivable};
use cosmwasm_std::{DepsMut, Env, MessageInfo};
use cw_ownable::initialize_owner;
use cw_storage_plus::{Item, Map};
use cw_tba::{Status, TokenInfo};
use saa_wasm::{saa_types::VerifiedData, save_verified};



pub static REGISTRY_ADDRESS: Item<String> = Item::new("r");
pub static TOKEN_INFO: Item<TokenInfo> = Item::new("t");
pub static STATUS: Item<Status> = Item::new("s");
pub static MINT_CACHE: Item<String> = Item::new("m");
pub static KNOWN_TOKENS: Map<(&str, &str), bool> = Map::new("k");


pub fn save_token_credentials(
    deps: &mut DepsMut,
    _env: &Env,
    mut info: MessageInfo,
    data: VerifiedData,
    owner: String,
) -> Result<(), ContractError> {

    // save the owner adderss to the storage
    info.sender = deps.api.addr_validate(&owner)?;
    initialize_owner(deps.storage, deps.api, Some(owner.as_str()))?;


    assert_data_owner_derivable(&data, owner)?;

    save_verified(deps.storage, data);
 
    Ok(())

}
