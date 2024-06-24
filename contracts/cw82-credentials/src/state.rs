use crate::{error::ContractError, msg::Status};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{ensure, Addr, DepsMut, Env, MessageInfo};
use cw_ownable::initialize_owner;
use cw_storage_plus::{Item, Map};
use cw_tba::TokenInfo;
use saa::{cosmos_utils::{pubkey_to_account, pubkey_to_canonical}, Binary, Credential, CredentialData, CredentialId, CredentialsWrapper, Verifiable};

#[cw_serde]
pub struct CredentialInfo {
    pub name: String,
    pub hrp: Option<String>,
    pub extra: Option<Binary>,
}

pub static REGISTRY_ADDRESS: Item<String> = Item::new("r");
pub static TOKEN_INFO: Item<TokenInfo> = Item::new("t");
pub static MINT_CACHE: Item<String> = Item::new("m");
pub static STATUS: Item<Status> = Item::new("s");
pub static SERIAL: Item<u128> = Item::new("l");

pub static VERIFYING_CRED_ID: Item<CredentialId> = Item::new("v");
pub static WITH_CALLER: Item<bool> = Item::new("w");
pub static SECS_TO_EXPIRE: Item<u64> = Item::new("e");

pub static CREDENTIALS: Map<CredentialId, CredentialInfo> = Map::new("c");
pub static KNOWN_TOKENS: Map<(&str, &str), bool> = Map::new("k");
pub static NONCES: Map<u128, bool> = Map::new("n");

/* const DEFAULT_SECS_TO_EXPIRE : u64 = 300;
const MIN_SECS_TO_EXPIRE     : u64 = 10;
 */

pub fn save_credentials(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    data: CredentialData,
    owner: String,
) -> Result<(), ContractError> {
    let with_caller = data.with_caller.unwrap_or_default();
    WITH_CALLER.save(deps.storage, &with_caller)?;

    let mut key_found = false;
    
    let mut owner_found = if with_caller { 
        owner == info.sender.to_string()
    } else { 
        false 
    };

    let info = if with_caller {
        MessageInfo {
            sender: Addr::unchecked(owner.clone()),
            ..info
        }
    } else {
        info.clone()
    };

    data.verified_cosmwasm(deps.api, &env, &Some(info))?;
    initialize_owner(deps.storage, deps.api, Some(owner.as_str()))?;
    

    if data.primary_index.is_some() {
        VERIFYING_CRED_ID.save(deps.storage, &data.primary_id())?;
        key_found = true;
    }

    for cred in data.credentials.iter() {
        let id: Vec<u8> = cred.value().id();

        let hrp = match cred {
            Credential::Caller(_) => {
                if String::from_utf8(id.clone()).unwrap() == owner {
                    owner_found = true;
                }
                None
            },
            cred => {
                if !key_found {
                    VERIFYING_CRED_ID.save(deps.storage, &id)?;
                    key_found = true;
                }
                if let Credential::CosmosArbitrary(c) = cred {
                    c.hrp.clone()
                } else if let Credential::Secp256k1(c) = cred {
                    c.hrp.clone()
                } else {
                    None
                }
            }
        };

        if !owner_found  {
            let addr = match hrp.as_ref() {
                Some(hrp) => pubkey_to_account(&id, hrp)?,
                None => deps.api.addr_humanize(&pubkey_to_canonical(&id))?.to_string()
            };

            if owner == addr {
                VERIFYING_CRED_ID.save(deps.storage, &id)?;
                owner_found = true;
            }
        }

        let hrp: Option<String> = match cred {
            Credential::CosmosArbitrary(c) => c.hrp.clone(),
            _ => None,
        };

        CREDENTIALS.save(
            deps.storage,
            id,
            &CredentialInfo {
                name: cred.name().to_string(),
                extra: None,
                hrp,
            },
        )?;
    }

    ensure!(owner_found, ContractError::NoOwnerCred {});

    Ok(())
}
