use std::borrow::BorrowMut;

use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError, StdResult
};
use cw_ownable::{get_ownership, is_owner};


#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw_tba::ExecuteAccountMsg;

use crate::{
    action::MINT_REPLY_ID,
    action,
    execute, 
    error::ContractError, 
    msg::{ContractResult, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, Status}, 
    query::{assets, can_execute, full_info, known_tokens, valid_signature, valid_signatures}, 
    state::{save_credentials, MINT_CACHE, REGISTRY_ADDRESS, SERIAL, STATUS, TOKEN_INFO, WITH_CALLER}, 
    utils::assert_valid_signed_actions 
};


#[cfg(target_arch = "wasm32")]
use crate::utils::query_if_registry;

pub const CONTRACT_NAME: &str = "crates:cw82-token-account";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(deps: DepsMut, env : Env, info : MessageInfo, msg : InstantiateMsg) -> ContractResult {

    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw22::set_contract_supported_interface(
        deps.storage, 
        &[
            cw22::ContractSupportedInterface {
                supported_interface: cw82::INTERFACE_NAME.into(),
                version: CONTRACT_VERSION.into()
            },
            cw22::ContractSupportedInterface {
                supported_interface: "crates:cw81".into(),
                version: CONTRACT_VERSION.into()
            },
            cw22::ContractSupportedInterface {
                supported_interface: "crates:cw1".into(),
                version: "1.1.1".into()
            },
        ]
    )?;
    #[cfg(target_arch = "wasm32")]
    if !query_if_registry(&deps.querier, info.sender.clone())? {
        return Err(ContractError::Unauthorized {})
    };
    
    TOKEN_INFO.save(deps.storage, &msg.token_info)?;
    REGISTRY_ADDRESS.save(deps.storage, &info.sender.to_string())?;

    STATUS.save(deps.storage, &Status { frozen: false })?;
    SERIAL.save(deps.storage, &0u128)?;
    
    save_credentials(deps, env, info, msg.account_data, msg.owner)?;

    Ok(Response::default())
}


pub fn execute_action<T, E, A>(
    deps       :   &mut DepsMut, 
    env        :   &Env, 
    info       :   &MessageInfo, 
    msg        :   ExecuteAccountMsg<T, E, A>,
) -> ContractResult {

    
    type Action<T, E, A> = ExecuteAccountMsg<T, E, A>;

    match msg {
        Action::MintToken { 
            minter: 
            collection, 
            msg 
        } => action::try_minting_token(deps, info, collection, msg),

        Action::TransferToken { 
            collection, 
            token_id, 
            recipient 
        } => action::try_transfering_token(deps, collection, token_id, recipient, info.funds.clone()),

        Action::SendToken { 
            collection, 
            token_id, 
            contract, 
            msg 
        } => action::try_sending_token(deps, collection, token_id, contract, msg, info.funds.clone()),

        Action::UpdateKnownTokens { 
            collection, 
            start_after, 
            limit 
        } => action::try_updating_known_tokens(
            deps, 
            env, 
            collection, 
            start_after, 
            limit
        ),
        Action::ForgetTokens { 
            collection, 
            token_ids 
        } => action::try_forgeting_tokens(
            deps, 
            collection, 
            token_ids
        ),

        Action::Freeze {} => action::try_freezing(deps),

        Action::Unfreeze {} => action::try_unfreezing(deps),
    
        _ => Err(ContractError::NotSupported {})
    }
}




#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(mut deps: DepsMut, env : Env, info : MessageInfo, msg : ExecuteMsg) -> ContractResult {
    if REGISTRY_ADDRESS.load(deps.storage).is_err() {
        return Err(ContractError::Deleted {})
    }
    SERIAL.update(deps.storage, |s| Ok::<u128, StdError>((s + 1) % u128::MAX))?;

    match msg {
        ExecuteMsg::Execute { 
            msgs 
        } => execute::try_executing(deps.as_ref(), env, info, msgs),

        ExecuteMsg::UpdateOwnership { 
            new_owner, 
            new_account_data 
        } => execute::try_updating_ownership(deps, env, info, new_owner, new_account_data),

        ExecuteMsg::UpdateAccountData { 
            new_account_data 
        } => execute::try_updating_account_data(deps, env, info, new_account_data),

        ExecuteMsg::ReceiveNft(
            msg
        ) => execute::try_updating_known_on_receive(deps, info.sender.to_string(), msg.token_id),

        ExecuteMsg::Purge {} => execute::try_purging(deps, info.sender),

        ExecuteMsg::Freeze {} => execute::try_freezing(deps),

        ExecuteMsg::Extension { 
            msg 
        } => {
            assert_valid_signed_actions(deps.as_ref(), &env, &msg)?;
            for action in msg.data.actions {
                execute_action(deps.borrow_mut(), &env, &info, action)?;
            }
            Ok(Response::default())
        },

        _ => {
            if WITH_CALLER.load(deps.storage)? && is_owner(deps.storage, &info.sender)? {
                execute_action(deps.borrow_mut(), &env, &info, msg)
            } else {
                Err(ContractError::Unauthorized {})
            }
        }
    }
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env : Env, msg: QueryMsg) -> StdResult<Binary> {
    if REGISTRY_ADDRESS.load(deps.storage).is_err() {
        return Err(StdError::GenericErr { 
            msg: ContractError::Deleted {}.to_string() 
        })
    };
    match msg {
        QueryMsg::Token {} => to_json_binary(&TOKEN_INFO.load(deps.storage)?),
        QueryMsg::Status {} => to_json_binary(&STATUS.load(deps.storage)?),
        QueryMsg::Serial {} => to_json_binary(&SERIAL.load(deps.storage)?),
        QueryMsg::Registry {} => to_json_binary(&REGISTRY_ADDRESS.load(deps.storage)?),
        QueryMsg::Ownership {} => to_json_binary(&get_ownership(deps.storage)?),
        QueryMsg::CanExecute { 
            sender, 
            msg 
        } => to_json_binary(&can_execute(deps, env, sender, msg)?),
        QueryMsg::ValidSignature { 
            signature, 
            data, 
            payload ,
        } => to_json_binary(&valid_signature(deps, data, signature, payload)?),
        QueryMsg::ValidSignatures { 
            signatures, 
            data, 
            payload 
        } => to_json_binary(&valid_signatures(deps, data, signatures, payload)?),
        QueryMsg::KnownTokens {
            skip,
            limit
        } => to_json_binary(&known_tokens(deps, skip, limit)?),
        QueryMsg::Assets {
            skip,
            limit
        } => to_json_binary(&assets(deps, env, skip, limit)?),
        QueryMsg::FullInfo {
            skip,
            limit
        } => to_json_binary(&full_info(deps, env, skip, limit)?),
        QueryMsg::Extension { 
            .. 
        } => to_json_binary(&String::from("signed_cosmos_msgs"))
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
                None
            )
        }
        _ => Err(ContractError::NotSupported {})
    }
}