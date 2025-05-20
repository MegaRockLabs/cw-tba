use std::vec;

use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError, StdResult
};
use cw_ownable::{get_ownership, initialize_owner};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw_tba::ExecuteMsg;
use cw_auths::handle_session_actions;
use strum::IntoDiscriminant;

use crate::{
    error::ContractError,
    execute::{
        try_changing_data, try_executing, try_fee_granting, try_forgeting_tokens, try_freezing, try_minting_token, try_purging, try_sending_token, try_transfering_token, try_unfreezing, try_updating_known_on_receive, try_updating_known_tokens, try_updating_ownership, MINT_REPLY_ID
    },
    msg::{InstantiateMsg, MigrateMsg, QueryMsg, Status},
    query::{assets, can_execute, full_info, known_tokens, valid_signature, valid_signatures},
    state::{MINT_CACHE, PUBKEY, REGISTRY_ADDRESS, SERIAL, STATUS, TOKEN_INFO}, utils::extract_pubkey,
};
#[cfg(target_arch = "wasm32")]
use crate::utils::query_if_registry;

pub const CONTRACT_NAME: &str = "crates:cw82-token-account";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
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

  
    let pubkey = extract_pubkey(deps.api, msg.account_data, &info.sender)?;

    initialize_owner(deps.storage, deps.api, Some(msg.owner.as_str()))?;
    TOKEN_INFO.save(deps.storage, &msg.token_info)?;
    REGISTRY_ADDRESS.save(deps.storage, &info.sender.to_string())?;
    STATUS.save(deps.storage, &Status { frozen: false })?;
    PUBKEY.save(deps.storage, &pubkey)?;
    SERIAL.save(deps.storage, &0u128)?;
    Ok(Response::default())
}



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    if REGISTRY_ADDRESS.load(deps.storage).is_err() {
        return Err(ContractError::Deleted {});
    }
    SERIAL.update(deps.storage, |s| Ok::<u128, StdError>((s + 1) % u128::MAX))?;
    let contract = env.contract.address.to_string();

    let (session, msg) = handle_session_actions(
        deps.api, 
        deps.storage, 
        &env, &info, 
        msg, 
        Some(contract)
    )?;

    let (msg, attrs) = if let Some(msg) = msg {
        let attrs = match session {
            Some(session) => vec![
                ("action", "with_session".to_string()),
                ("session_key", session.key()),
                ("nonce", session.nonce.to_string()),
            ],
            None => vec![]
        };
        (msg, attrs)
    } else {
        return Ok(match session {
            Some(session) => Response::new()
                .add_attribute("action", "session_created")
                .add_attribute("session_key", session.key().as_str())
                .add_attribute("nonce", session.nonce.to_string().as_str()),

            None => Response::new()
                .add_attribute("action", "session_revoked"),
        })
    };

    let name = msg.discriminant().to_string();

    
    let res = match msg {
        ExecuteMsg::Execute { msgs } => try_executing(deps.as_ref(), info.sender, msgs),
        ExecuteMsg::MintToken {
            minter: collection,
            msg,
        } => try_minting_token(deps, info.sender, collection, msg, info.funds),
        ExecuteMsg::TransferToken {
            collection,
            token_id,
            recipient,
        } => try_transfering_token(deps, collection, token_id, recipient, info.funds),
        ExecuteMsg::SendToken {
            collection,
            token_id,
            contract,
            msg,
        } => try_sending_token(deps, collection, token_id, contract, msg, info.funds),
        ExecuteMsg::UpdateKnownTokens {
            collection,
            start_after,
            limit,
        } => try_updating_known_tokens(deps, env, info.sender, collection, start_after, limit),
        ExecuteMsg::Freeze {} => try_freezing(deps, info.sender),
        ExecuteMsg::Unfreeze {} => try_unfreezing(deps),
        ExecuteMsg::ForgetTokens {
            collection,
            token_ids,
        } => try_forgeting_tokens(deps, info.sender, collection, token_ids),
        ExecuteMsg::ReceiveNft(msg) => {
            try_updating_known_on_receive(deps, info.sender.to_string(), msg.token_id)
        }
        ExecuteMsg::UpdateOwnership {
            new_owner,
            new_account_data,
        } => try_updating_ownership(deps, env, info, new_owner, new_account_data),
        ExecuteMsg::UpdateAccountData(op) => try_changing_data(deps, env, info, op),
        ExecuteMsg::Purge {} => try_purging(deps, info.sender),
        ExecuteMsg::FeeGrant { 
            grantee, 
            allowance 
        } => try_fee_granting(deps, env.contract.address, info.sender, grantee, allowance),

        _ => unreachable!(),
    }?;

    Ok(res
        .add_attribute("action_name", name)
        .add_attributes(attrs))
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    if REGISTRY_ADDRESS.load(deps.storage).is_err() {
        return Err(StdError::generic_err(ContractError::Deleted {}.to_string()));
    };
    if let Some(res) = cw_auths::handle_session_queries(deps.api, deps.storage, &env, &msg)? {
        return Ok(res);
    }
    match msg {
        QueryMsg::Token {} => to_json_binary(&TOKEN_INFO.load(deps.storage)?),
        QueryMsg::Status {} => to_json_binary(&STATUS.load(deps.storage)?),
        QueryMsg::Serial {} => to_json_binary(&SERIAL.load(deps.storage)?),
        QueryMsg::Pubkey {} => to_json_binary(&PUBKEY.load(deps.storage)?),
        QueryMsg::Registry {} => to_json_binary(&REGISTRY_ADDRESS.load(deps.storage)?),
        QueryMsg::Ownership {} => to_json_binary(&get_ownership(deps.storage)?),
        QueryMsg::CanExecute { 
            sender, 
            msg 
        } => to_json_binary(&can_execute(deps, sender, &msg)?),
        QueryMsg::ValidSignature {
                        signature,
                        data,
                        payload,
            } => to_json_binary(&valid_signature(deps, data, signature, &payload)?),
        QueryMsg::ValidSignatures {
                signatures,
                data,
                payload,
            } => to_json_binary(&valid_signatures(deps, data, signatures, &payload)?),
        QueryMsg::KnownTokens { skip, limit } => to_json_binary(&known_tokens(deps, skip, limit)?),
        QueryMsg::Assets { skip, limit } => to_json_binary(&assets(deps, env, skip, limit)?),
        QueryMsg::FullInfo { skip, limit } => to_json_binary(&full_info(deps, env, skip, limit)?),
        QueryMsg::SessionQueries(_) => unreachable!(),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _: Env, _: MigrateMsg) -> StdResult<Response> {
    STATUS.save(deps.storage, &Status { frozen: false })?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        MINT_REPLY_ID => {
            let collection = MINT_CACHE.load(deps.storage)?;
            MINT_CACHE.remove(deps.storage);
            // query all the held tokens for the collection stored in CACHE
            try_updating_known_tokens(
                deps,
                env.clone(),
                env.contract.address,
                collection.to_string(),
                None,
                None,
            )
        }
        _ => Err(ContractError::NotSupported {}),
    }
}