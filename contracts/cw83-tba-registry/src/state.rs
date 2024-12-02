use cw_storage_plus::{Item, Map};
use cw_tba::RegistryParams;

/// A Mapping of the collections addresses to the number of a tokens with an account
pub static COL_TOKEN_COUNTS: Map<&str, u32> = Map::new("c");
/// A Mapping where (collection_address, token_id) => token-bound account address
pub static TOKEN_ADDRESSES: Map<(&str, &str), String> = Map::new("t");
/// Registry params
pub static REGISTRY_PARAMS: Item<RegistryParams> = Item::new("p");
