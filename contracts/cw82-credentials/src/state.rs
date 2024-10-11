use crate::error::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{ensure, Addr, Api, Binary, Env, MessageInfo, Storage};
use cw_ownable::initialize_owner;
use cw_storage_plus::{Item, Map};
use cw_tba::{Status, TokenInfo};
use saa::{
    cosmos_utils::{pubkey_to_account, pubkey_to_canonical}, 
    Credential, CredentialData, CredentialId, CredentialsWrapper, 
    PasskeyCredential, Verifiable
};

#[cw_serde]
pub struct CredentialInfo<E = Binary> {
    pub name: String,
    pub hrp: Option<String>,
    pub extension: Option<E>,
}


pub static REGISTRY_ADDRESS: Item<String> = Item::new("r");
pub static TOKEN_INFO: Item<TokenInfo> = Item::new("t");
pub static MINT_CACHE: Item<String> = Item::new("m");
pub static STATUS: Item<Status> = Item::new("s");
pub static SERIAL: Item<u128> = Item::new("l");

pub static VERIFYING_CRED_ID: Item<CredentialId> = Item::new("v");
pub static WITH_CALLER: Item<bool> = Item::new("w");
pub static CREDENTIALS: Map<CredentialId, CredentialInfo> = Map::new("c");
pub static NONCES: Map<u128, bool> = Map::new("n");

pub static KNOWN_TOKENS: Map<(&str, &str), bool> = Map::new("k");


pub fn save_credentials(
    api: &dyn Api,
    storage: &mut dyn Storage,
    env: &Env,
    info: MessageInfo,
    data: CredentialData,
    owner: String,
) -> Result<(), ContractError> {
    let with_caller = data.with_caller.unwrap_or_default();
    WITH_CALLER.save(storage, &with_caller)?;
    
    let mut verifying_found = false;
    let mut owner_found = false;

    let info = if with_caller {
        owner_found = true;
        MessageInfo {
            sender: Addr::unchecked(owner.clone()),
            ..info
        }
    } else {
        info.clone()
    };

    let data = data.verified_cosmwasm(api, env, &Some(info))?;
    initialize_owner(storage, api, Some(owner.as_str()))?;
    

    if data.primary_index.is_some() {
        if let Credential::Caller(_) = data.primary() {} else {
            VERIFYING_CRED_ID.save(storage, &data.primary_id())?;
            verifying_found = true;
        }
    }

    CREDENTIALS.clear(storage);

    for cred in data.credentials.iter() {
        let id: Vec<u8> = cred.value().id();
  
        let hrp : Option<String> = match cred {
            Credential::CosmosArbitrary(c) => c.hrp.clone(),
            Credential::Secp256k1(c) => c.hrp.clone(),
            _ => None
        };

        let extension : Option<Binary> = match cred {
            Credential::Passkey(PasskeyCredential {
                pubkey,
                ..
            }) => {
                ensure!(pubkey.is_some(), ContractError::Generic("Public key is required for passkeys".to_string()));
                Some(pubkey.clone().unwrap().into())
            }
            _ => None
        };
     
        if !owner_found  {
            let addr = match hrp.as_ref() {
                Some(hrp) => pubkey_to_account(&id, hrp)?,
                None => api.addr_humanize(&pubkey_to_canonical(&id))?.to_string()
            };
            if owner == addr {
                owner_found = true;
                // if primary not specified explicitly, override one derived into the owner to be primary
                if data.primary_index.is_none() {
                    VERIFYING_CRED_ID.save(storage, &id)?;
                    verifying_found = true;
                }
            }
        }

        // first non-caller credential is the verifying one
        if !verifying_found {
            if let Credential::Caller(_) = data.primary() {} else {
                VERIFYING_CRED_ID.save(storage, &data.primary_id())?;
                verifying_found = true;
            }
        };

        CREDENTIALS.save(
            storage,
            id,
            &CredentialInfo {
                name: cred.name().to_string(),
                hrp,
                extension,
            },
        )?;
    }

    ensure!(owner_found, ContractError::NoOwnerCred {});
    ensure!(verifying_found, ContractError::NoVerifyingCred {});

    Ok(())
}
