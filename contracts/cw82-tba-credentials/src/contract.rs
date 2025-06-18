#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
#[cfg(target_arch = "wasm32")]
use crate::utils::query_if_registry;
#[cfg(feature = "archway")]
use {cosmwasm_std::SubMsg, crate::grants::register_granter_msg};
use saa_wasm::{account_number, verify_native};


use cosmwasm_std::{
    to_json_binary, Binary, 
    Deps, DepsMut, Env, MessageInfo, Reply, Response, 
    StdError, StdResult
};

use cw_ownable::get_ownership;
use cw_tba::{ExecuteMsg, Status};


use crate::{
    action::{self, MINT_REPLY_ID}, 
    error::ContractError, execute::{self, try_executing_actions}, 
    msg::{ContractResult, InstantiateMsg, MigrateMsg, QueryMsg}, 
    query::{assets, can_execute, can_execute_signed, full_info, known_tokens, 
        valid_signature, valid_signatures /* valid_signatures */
    }, 
    state::{save_token_credentials, MINT_CACHE, REGISTRY_ADDRESS, STATUS, TOKEN_INFO}
};


pub const CONTRACT_NAME: &str = "crates:cw82-credential-token-account";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult {
    #[cfg(target_arch = "wasm32")]
    if !query_if_registry(&deps.querier, info.sender.clone())? {
        return Err(ContractError::NotRegistry {});
    };
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw22::set_contract_supported_interface(
        deps.storage,
        &[
            cw22::ContractSupportedInterface {
                supported_interface: "crates:cw84".into(),
                version: CONTRACT_VERSION.into(),
            },
            cw22::ContractSupportedInterface {
                supported_interface: "crates:cw82".into(),
                version: CONTRACT_VERSION.into(),
            },
            cw22::ContractSupportedInterface {
                supported_interface: "crates:cw81".into(),
                version: CONTRACT_VERSION.into(),
            },
            cw22::ContractSupportedInterface {
                supported_interface: "crates:cw1".into(),
                version: CONTRACT_VERSION.into(),
            },
        ],
    )?;
    TOKEN_INFO.save(deps.storage, &msg.token_info)?;
    REGISTRY_ADDRESS.save(deps.storage, &info.sender.to_string())?;
    STATUS.save(deps.storage, &Status { frozen: false })?;

    save_token_credentials(deps.api, deps.storage, msg.account_data, msg.owner.as_str())?;
    let actions = msg.actions.unwrap_or_default();
    let res = try_executing_actions(deps, &env, info, actions)?;

    #[cfg(feature = "archway")]
    let res = res.add_submessage(SubMsg::new(register_granter_msg(&env)?));
    Ok(res)

}



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> ContractResult {
    if REGISTRY_ADDRESS.load(deps.storage).is_err() {
        return Err(ContractError::Deleted {});
    }
    // let contract = env.contract.address.to_string();
    // let action_name = msg.discriminant().to_string();

    let res = match msg {

        ExecuteMsg::ExecuteSigned { 
            msgs, 
            signed 
        } => execute::try_executing_signed(deps, env, info, msgs, signed),

        ExecuteMsg::ExecuteNative { 
            msgs 
        } => execute::try_executing_actions(deps, &env, info, msgs),

        ExecuteMsg::UpdateOwnership {
            new_owner,
            new_account_data,
        } => execute::try_updating_ownership(deps, env, info, new_owner, new_account_data),

        ExecuteMsg::UpdateAccountData(op) => {
            execute::try_updating_account_data(deps, env, info, op)
        }

        ExecuteMsg::ReceiveNft(msg) => {
            execute::try_updating_known_on_receive(deps, info.sender.to_string(), msg.token_id)
        }

   /*      ExecuteMsg::SessionActions(
            session_action_msg
        ) => handle_session_action(
                deps, 
                &env, 
                &info, 
                session_action_msg, 
                Some(contract),
                execute_action
        ), */

        ExecuteMsg::Execute { msgs } => {
            verify_native(deps.storage, info.sender.to_string())?;
            action::try_executing(msgs)
        },

        ExecuteMsg::Purge {} => execute::try_purging(deps.api, deps.storage, info.sender.as_str()),

        ExecuteMsg::Freeze {} => execute::try_freezing(deps),
    }?;

    Ok(res
    //    .add_attribute("action_name", action_name)
    )
}



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    if REGISTRY_ADDRESS.load(deps.storage).is_err() {
        return Err(StdError::generic_err(ContractError::Deleted {}.to_string()));
    };
    match msg {
        QueryMsg::Token {} => to_json_binary(&TOKEN_INFO.load(deps.storage)?),
        QueryMsg::Status {} => to_json_binary(&STATUS.load(deps.storage)?),
        QueryMsg::AccountNumber {} => to_json_binary(&account_number(deps.storage)),
        QueryMsg::Registry {} => to_json_binary(&REGISTRY_ADDRESS.load(deps.storage)?),
        QueryMsg::Ownership {} => to_json_binary(&get_ownership(deps.storage)?),
        QueryMsg::KnownTokens { skip, limit } => to_json_binary(&known_tokens(deps, skip, limit)?),
        QueryMsg::Assets { skip, limit } => to_json_binary(&assets(deps, env, skip, limit)?),
        QueryMsg::FullInfo { skip, limit } => to_json_binary(&full_info(deps, env, skip, limit)?),
        // QueryMsg::SessionQueries(q) => handle_session_query(deps.api, deps.storage, &env, q),
        QueryMsg::CanExecute { 
            sender, 
            msg 
        } => to_json_binary(&can_execute(deps, sender, msg)?),
        QueryMsg::CanExecuteSigned { 
            msgs, 
            signed 
        } => to_json_binary(&can_execute_signed(deps, env,  msgs, signed)?),
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
    }
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _: Env, _: MigrateMsg) -> ContractResult {
    STATUS.save(deps.storage, &Status { frozen: false })?;
    Ok(Response::default())
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> ContractResult {
    match msg.id {
        MINT_REPLY_ID => {
            let collection = MINT_CACHE.load(deps.storage)?;
            MINT_CACHE.remove(deps.storage);
            // query all the held tokens for the collection stored in CACHE
            action::try_updating_known_tokens(
                &deps.querier,
                deps.storage,
                &env,
                collection,
                None,
                None,
            )
        }
        _ => Err(ContractError::NotSupported {}),
    }
}


#[cfg(feature = "archway")]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(
    deps: DepsMut,
    env: Env,
    msg: crate::msg::SudoMsg,
) -> ContractResult {
    return match msg {
        crate::msg::SudoMsg::CwGrant(grant) => { 
            crate::grants::cwfee_grant(deps, env, grant)
        }
    }
}