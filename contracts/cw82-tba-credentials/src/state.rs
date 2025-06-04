use crate::{error::ContractError, utils::assert_owner_derivable};
use cosmwasm_std::{ContractInfo, DepsMut, Env, MessageInfo};
use cw_ownable::initialize_owner;
use cw_storage_plus::{Item, Map};
use cw_tba::{Status, TokenInfo};
use saa_wasm::{save_credentials, saa_types::CredentialData};



pub static REGISTRY_ADDRESS: Item<String> = Item::new("r");
pub static TOKEN_INFO: Item<TokenInfo> = Item::new("t");
pub static STATUS: Item<Status> = Item::new("s");
pub static MINT_CACHE: Item<String> = Item::new("m");
pub static KNOWN_TOKENS: Map<(&str, &str), bool> = Map::new("k");


pub fn save_token_credentials(
    deps: &mut DepsMut,
    env: &Env,
    mut info: MessageInfo,
    data: CredentialData,
    owner: String,
) -> Result<(), ContractError> {

    // On account creatino the signed contract address that of the registry
    // instead of the account contract
    let registry_env = Env {
        contract: ContractInfo { address: info.sender.clone() },
        ..env.clone()
    };

    // save the owner adderss to the storage
    info.sender = deps.api.addr_validate(&owner)?;
    initialize_owner(deps.storage, deps.api, Some(owner.as_str()))?;

    let saved = save_credentials(
        deps,
        &registry_env,
        &info,
        &data,
    )?;

    assert_owner_derivable(saved, owner)
}
