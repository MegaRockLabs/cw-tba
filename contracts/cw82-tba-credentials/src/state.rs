use crate::{error::ContractError, utils::assert_data_owner_derivable};
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
    api: &dyn cosmwasm_std::Api,
    storage: &mut dyn cosmwasm_std::Storage,
    data: VerifiedData,
    owner: &str,
) -> Result<(), ContractError> {
    assert_data_owner_derivable(&data, owner)?;
    initialize_owner(storage, api, Some(owner))?;
    save_verified(storage, data)?;
    Ok(())

}
