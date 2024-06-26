use std::borrow::BorrowMut;

use cosmwasm_std::{
    from_json, to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError, StdResult
};
use cw_ownable::get_ownership;

#[cfg(feature = "archway")]
use crate::grants;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw_tba::Status;

use crate::{
    execute,  
    error::ContractError, 
    action::{self, MINT_REPLY_ID}, 
    msg::{ContractResult, CredQueryMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg}, 
    query::{assets, can_execute, credentials, full_info, known_tokens, valid_signature, valid_signatures}, 
    state::{save_credentials, MINT_CACHE, REGISTRY_ADDRESS, SERIAL, STATUS, TOKEN_INFO}
};

#[cfg(target_arch = "wasm32")]
use crate::utils::query_if_registry;

pub const CONTRACT_NAME: &str = "crates:cw82-credential-token-account";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw22::set_contract_supported_interface(
        deps.storage,
        &[
            cw22::ContractSupportedInterface {
                supported_interface: cw82::INTERFACE_NAME.into(),
                version: CONTRACT_VERSION.into(),
            },
            cw22::ContractSupportedInterface {
                supported_interface: "crates:cw81".into(),
                version: CONTRACT_VERSION.into(),
            },
            cw22::ContractSupportedInterface {
                supported_interface: "crates:cw1".into(),
                version: "1.1.1".into(),
            },
        ],
    )?;
    #[cfg(target_arch = "wasm32")]
    if !query_if_registry(&deps.querier, info.sender.clone())? {
        return Err(ContractError::Unauthorized {});
    };

    TOKEN_INFO.save(deps.storage, &msg.token_info)?;
    REGISTRY_ADDRESS.save(deps.storage, &info.sender.to_string())?;

    STATUS.save(deps.storage, &Status { frozen: false })?;
    SERIAL.save(deps.storage, &0u128)?;

    save_credentials(deps, env.clone(), info, from_json(&msg.account_data)?, msg.owner)?;

    let mut msgs: Vec<CosmosMsg> = Vec::with_capacity(1);
    #[cfg(feature = "archway")]
    msgs.push(grants::register_granter_msg(&env)?);
    

    Ok(Response::default().add_messages(msgs))
}



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> ContractResult {
    if REGISTRY_ADDRESS.load(deps.storage).is_err() {
        return Err(ContractError::Deleted {});
    }
    SERIAL.update(deps.storage, |s| Ok::<u128, StdError>((s + 1) % u128::MAX))?;

    match msg {
        ExecuteMsg::Execute { 
            msgs 
        } => execute::try_executing(deps, env, info, msgs),

        ExecuteMsg::UpdateOwnership {
            new_owner,
            new_account_data,
        } => execute::try_updating_ownership(deps, env, info, new_owner, new_account_data),

        ExecuteMsg::UpdateAccountData { new_account_data } => {
            execute::try_updating_account_data(deps, env, info, new_account_data)
        }

        ExecuteMsg::ReceiveNft(msg) => {
            execute::try_updating_known_on_receive(deps, info.sender.to_string(), msg.token_id)
        }

        ExecuteMsg::Purge {} => execute::try_purging(deps, info.sender),

        ExecuteMsg::Freeze {} => execute::try_freezing(deps),

        ExecuteMsg::Extension { .. } => Ok(Response::default()),

        _ => Err(ContractError::NotSupported {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    if REGISTRY_ADDRESS.load(deps.storage).is_err() {
        return Err(StdError::GenericErr {
            msg: ContractError::Deleted {}.to_string(),
        });
    };
    match msg {
        QueryMsg::Token {} => to_json_binary(&TOKEN_INFO.load(deps.storage)?),
        QueryMsg::Status {} => to_json_binary(&STATUS.load(deps.storage)?),
        QueryMsg::Serial {} => to_json_binary(&SERIAL.load(deps.storage)?),
        QueryMsg::Registry {} => to_json_binary(&REGISTRY_ADDRESS.load(deps.storage)?),
        QueryMsg::Ownership {} => to_json_binary(&get_ownership(deps.storage)?),
        QueryMsg::CanExecute { sender, msg } => {
            to_json_binary(&can_execute(deps, env, sender, msg)?)
        }
        QueryMsg::ValidSignature {
            signature,
            data,
            payload,
        } => to_json_binary(&valid_signature(deps, env, data, signature, payload)?),
        QueryMsg::ValidSignatures {
            signatures,
            data,
            payload,
        } => to_json_binary(&valid_signatures(deps, env, data, signatures, payload)?),
        QueryMsg::KnownTokens { skip, limit } => to_json_binary(&known_tokens(deps, skip, limit)?),
        QueryMsg::Assets { skip, limit } => to_json_binary(&assets(deps, env, skip, limit)?),
        
        QueryMsg::Extension { msg } => {
            match msg {
                CredQueryMsg::FullInfo { skip, limit } => to_json_binary(&full_info(deps, env, skip, limit)?),
                CredQueryMsg::Credentials {} => to_json_binary(&credentials(deps)?)
            }
        },
    }
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _: Env, _: MigrateMsg) -> ContractResult {
    STATUS.save(deps.storage, &Status { frozen: false })?;
    Ok(Response::default())
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(mut deps: DepsMut, env: Env, msg: Reply) -> ContractResult {
    match msg.id {
        MINT_REPLY_ID => {
            let collection = MINT_CACHE.load(deps.storage)?;
            MINT_CACHE.remove(deps.storage);
            // query all the held tokens for the collection stored in CACHE
            action::try_updating_known_tokens(
                deps.borrow_mut(),
                &env,
                collection.to_string(),
                None,
                None,
            )
        }
        _ => Err(ContractError::NotSupported {}),
    }
}



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(
    deps: DepsMut,
    env: Env,
    msg: crate::grants::SudoMsg,
) -> Result<Response, ContractError> {
    return match msg {
        crate::grants::SudoMsg::CwGrant(grant) => {
            crate::grants::sudo_grant(deps, env, grant)
        }
    }
}