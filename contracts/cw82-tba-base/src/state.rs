use cosmwasm_std::Binary;
use cw_storage_plus::{Item, Map};
use cw_tba::TokenInfo;

use crate::msg::Status;

pub static REGISTRY_ADDRESS: Item<String> = Item::new("r");
pub static TOKEN_INFO: Item<TokenInfo> = Item::new("t");
pub static STATUS: Item<Status> = Item::new("s");
pub static PUBKEY: Item<Binary> = Item::new("p");
pub static MINT_CACHE: Item<String> = Item::new("m");
pub static SERIAL: Item<u128> = Item::new("l");

pub static KNOWN_TOKENS: Map<(&str, &str), bool> = Map::new("k");
