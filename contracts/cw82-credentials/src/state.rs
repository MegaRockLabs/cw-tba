use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, DepsMut, Env, MessageInfo};
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
pub static STATUS             : Item<Status>      = Item::new("s");
pub static TOKEN_INFO         : Item<TokenInfo>   = Item::new("t");
pub static MINT_CACHE         : Item<String>      = Item::new("m");
pub static SERIAL             : Item<u128>        = Item::new("l");


pub static VERIFYING_CRED_ID  : Item<CredentialId>        = Item::new("v");
pub static WITH_CALLER        : Item<bool>                = Item::new("w");


pub static CREDENTIALS        : Map<CredentialId, CredentialInfo> = Map::new("c"); 
pub static KNOWN_TOKENS       : Map<(&str, &str), bool>           = Map::new("k");



pub fn save_credentials(
    deps      :     DepsMut,
    env       :     Env,
    info      :     MessageInfo,
    data      :     CredentialData,
    owner     :     String
) -> Result<(), ContractError> {

    let with_caller = data.with_caller.unwrap_or_default();
    WITH_CALLER.save(deps.storage, &with_caller)?;

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
        if !key_found {
            match cred {
                // doesn't have a key or anything to check signature
                Credential::Caller(_) => {},
                // all currently supported credentials have a public key as id
                _ => {
                    VERIFYING_CRED_ID.save(
                        deps.storage, 
                        &cred.value().id()
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
            cred.value().id(),
            &CredentialInfo {
                name: cred.name().to_string(),
                extra: None,
                hrp,
            }
        )?;
    }
    Ok(())
}
