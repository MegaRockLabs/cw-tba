use cosmwasm_schema::cw_serde;
use cosmwasm_std::{ensure, Addr, Binary, DepsMut, Env, MessageInfo};
use cw_ownable::initialize_owner;
use cw_storage_plus::{Item, Map};
use cw_tba::TokenInfo;
use saa::{Credential, CredentialData, CredentialId, CredentialsWrapper};
use saa::Verifiable;

use crate::{error::ContractError, msg::Status};

#[cw_serde]
pub struct CredentialInfo {
    pub name    : String,
    pub hrp     : Option<String>,
    pub extra   : Option<Binary>
}


pub static REGISTRY_ADDRESS   : Item<String>      = Item::new("r");
pub static TOKEN_INFO         : Item<TokenInfo>   = Item::new("t");
pub static MINT_CACHE         : Item<String>      = Item::new("m");
pub static STATUS             : Item<Status>      = Item::new("s");
pub static SERIAL             : Item<u128>        = Item::new("l");


pub static VERIFYING_CRED_ID  : Item<CredentialId>        = Item::new("v");
pub static WITH_CALLER        : Item<bool>                = Item::new("w");
pub static SECS_TO_EXPIRE     : Item<u64>                 = Item::new("e");

pub static CREDENTIALS        : Map<CredentialId, CredentialInfo> = Map::new("c"); 
pub static KNOWN_TOKENS       : Map<(&str, &str), bool>           = Map::new("k");


const DEFAULT_SECS_TO_EXPIRE : u64 = 300;
const MIN_SECS_TO_EXPIRE     : u64 = 10;

pub fn save_credentials(
    deps      :     DepsMut,
    env       :     Env,
    info      :     MessageInfo,
    data      :     CredentialData,
    owner     :     String
) -> Result<(), ContractError> {

    let with_caller = data.with_caller.unwrap_or_default();
    WITH_CALLER.save(deps.storage, &with_caller)?;

    let secs_to_expire = data.secs_to_expire.unwrap_or(DEFAULT_SECS_TO_EXPIRE);
    ensure!(
        secs_to_expire > MIN_SECS_TO_EXPIRE, 
        ContractError::Generic("secs_to_expire must be greater than 10".to_string())
    );
    SECS_TO_EXPIRE.save(deps.storage, &data.secs_to_expire.unwrap_or(DEFAULT_SECS_TO_EXPIRE))?;


    let info = if with_caller {
        MessageInfo {
            sender: Addr::unchecked(owner.clone()),
            ..info
        }
    } else {
        info.clone()
    };

    data.verified_cosmwasm(deps.api, &env, &info)?;
    initialize_owner(deps.storage, deps.api, Some(owner.as_str()))?;


    let mut key_found = false;

    if data.primary_index.is_some() {
        VERIFYING_CRED_ID.save(deps.storage, &data.primary_id())?;
        key_found = true;
    }

    for cred in data.credentials.iter() {

        let id: Vec<u8> = cred.value().id();

        if !key_found {
            match cred {
                // doesn't have a key or anything to check the signature
                Credential::Caller(_) => {},
                // all currently supported credentials have a public key as id. Save the first one
                _ => {
                    VERIFYING_CRED_ID.save(
                        deps.storage, 
                        &id
                    )?;
                    key_found = true;
                }
            }
        }

        let hrp: Option<String> = match cred {
            Credential::CosmosArbitrary(c) => c.hrp.clone(),
            _ => None
        };


        CREDENTIALS.save(
            deps.storage, 
            id,
            &CredentialInfo {
                name: cred.name().to_string(),
                extra: None,
                hrp,
            }
        )?;
    }
    Ok(())
}