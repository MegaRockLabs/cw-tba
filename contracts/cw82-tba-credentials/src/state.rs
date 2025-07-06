use crate::error::ContractError;
use cw_storage_plus::{Item, Map};
use cw_tba::{Status, TokenInfo};
use saa_wasm::saa_types::VerifiedData;

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
    crate::utils::assert_owner_derivable(&data.credentials, owner)?;
    cw_ownable::initialize_owner(storage, api, Some(owner))?;
    saa_wasm::save_credentials(storage, &data)?;
    Ok(())
}
