use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, MessageInfo, StdResult, Response, Reply, StdError, to_json_binary,
};
use cw_ownable::{get_ownership, initialize_owner};


#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::{
    state::{REGISTRY_ADDRESS, TOKEN_INFO, PUBKEY, STATUS, MINT_CACHE, SERIAL}, 
    msg::{QueryMsg, InstantiateMsg, ExecuteMsg, Status, MigrateMsg}, 
    error::ContractError, 
    query::{can_execute, valid_signature, valid_signatures, known_tokens, assets, full_info}, 
    execute::{
        try_executing, 
        try_updating_ownership, 
        try_updating_known_tokens, 
        try_forgeting_tokens, 
        try_updating_known_on_receive, 
        try_transfering_token, 
        try_sending_token, 
        try_freezing, 
        try_unfreezing, 
        try_changing_pubkey, 
        try_minting_token, 
        try_purging,
        MINT_REPLY_ID
    }, 
};

#[cfg(target_arch = "wasm32")]
use crate::utils::query_if_registry;

pub const CONTRACT_NAME: &str = "crates:cw82-token-account";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(deps: DepsMut, _ : Env, info : MessageInfo, msg : InstantiateMsg) 
-> Result<Response, ContractError> {
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
    initialize_owner(deps.storage, deps.api, Some(msg.owner.as_str()))?;
    TOKEN_INFO.save(deps.storage, &msg.token_info)?;
    REGISTRY_ADDRESS.save(deps.storage, &info.sender.to_string())?;
    STATUS.save(deps.storage, &Status { frozen: false })?;
    PUBKEY.save(deps.storage, &msg.account_data)?;
    SERIAL.save(deps.storage, &0u128)?;
    Ok(Response::default())
}



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env : Env, info : MessageInfo, msg : ExecuteMsg) 
-> Result<Response, ContractError> {
    if REGISTRY_ADDRESS.load(deps.storage).is_err() {
        return Err(ContractError::Deleted {})
    }
    SERIAL.update(deps.storage, |s| Ok::<u128, StdError>((s + 1) % u128::MAX))?;

    match msg {
        ExecuteMsg::Execute { msgs } => try_executing(deps.as_ref(), info.sender, msgs),
        ExecuteMsg::MintToken { 
            minter: 
            collection, 
            msg 
        } => try_minting_token(deps, info.sender, collection, msg, info.funds),
        ExecuteMsg::TransferToken { 
            collection, 
            token_id, 
            recipient 
        } => try_transfering_token(deps, collection, token_id, recipient, info.funds),
        ExecuteMsg::SendToken { 
            collection, 
            token_id, 
            contract, 
            msg 
        } => try_sending_token(deps, collection, token_id, contract, msg, info.funds),
        ExecuteMsg::UpdateKnownTokens { 
            collection, 
            start_after, 
            limit 
        } => try_updating_known_tokens(
            deps, 
            env, 
            info.sender, 
            collection, 
            start_after, 
            limit
        ),
        ExecuteMsg::Freeze {} => try_freezing(deps, info.sender),
        ExecuteMsg::Unfreeze {} => try_unfreezing(deps),
        ExecuteMsg::ForgetTokens { 
            collection, 
            token_ids 
        } => try_forgeting_tokens(deps, info.sender, collection, token_ids),
        ExecuteMsg::ReceiveNft(
            msg
        ) => try_updating_known_on_receive(deps, info.sender.to_string(), msg.token_id),
        ExecuteMsg::UpdateOwnership { 
            new_owner, 
            new_account_data 
        } => try_updating_ownership(deps, info.sender, new_owner, new_account_data),
        ExecuteMsg::UpdateAccountData { new_account_data } => try_changing_pubkey(deps, info.sender, new_account_data),
        ExecuteMsg::Purge {} => try_purging(deps, info.sender),
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
            payload ,
        } => to_json_binary(&valid_signature(deps, data, signature, &payload)?),
        QueryMsg::ValidSignatures { 
            signatures, 
            data, 
            payload 
        } => to_json_binary(&valid_signatures(deps, data, signatures, &payload)?),
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
        } => Ok(Binary::default())
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
                None
            )
        }
        _ => Err(ContractError::NotSupported {})
    }
}