use crate::{
    action::execute_action, error::ContractError, msg::ContractResult, state::{
        save_token_credentials, KNOWN_TOKENS, REGISTRY_ADDRESS, STATUS, TOKEN_INFO
    }, utils::{assert_owner_derivable, assert_registry, assert_status}
};
use cosmwasm_std::{ensure, Api, DepsMut, Env, MessageInfo, Response, Storage};
use cw2::CONTRACT;
use cw22::SUPPORTED_INTERFACES;
use cw_ownable::get_ownership;
use cw_tba::{verify_nft_ownership, ExecuteAccountMsg, Status};
use saa_wasm::{ 
    add_credentials, remove_credentials, saa_types::{msgs::SignedDataMsg, CredentialData}, verify_signed_actions, UpdateOperation
};


pub fn try_executing_signed(
    mut deps :  DepsMut,
    env      :   &Env,
    info     :   MessageInfo,
    msgs     :   Vec<ExecuteAccountMsg>,
    signed   :   SignedDataMsg,
) -> ContractResult {
    assert_status(deps.storage)?;
    verify_signed_actions(&mut deps, &env, msgs.clone(), signed)?;
    try_executing_actions(deps, env, info, msgs)
}


pub fn try_executing_actions(
    mut deps  :   DepsMut,
    env       :   &Env,
    info      :   MessageInfo,
    actions   :   Vec<ExecuteAccountMsg>,
) -> ContractResult {
    let mut res = Response::new();
    for act in actions {
        let action_res = execute_action(&mut deps, &env, &info, act)?;
        res = res
            .add_submessages(action_res.messages)
            .add_events(action_res.events)
            .add_attributes(action_res.attributes);
        if let Some(data) = action_res.data {
            res = res.set_data(data);
        }
    }
    Ok(res)
}



pub fn try_updating_account_data(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    op: UpdateOperation
) -> ContractResult {
    let ownership = cw_ownable::get_ownership(deps.storage)?;
    let owner = ownership.owner.unwrap();

    match op {
        UpdateOperation::Add(data) => {
            // let data = data.with_native(&info);
            let added = add_credentials(&mut deps, &env, data)?;  
            if let Some(pending) = ownership.pending_owner {
                STATUS.save(deps.storage, &Status { frozen: false })?;
                assert_owner_derivable(added, pending.to_string())?;
                cw_ownable::update_ownership(deps, &env.block, &pending, cw_ownable::Action::AcceptOwnership)?; 
            }
        },
        UpdateOperation::Remove(idx) => {
            let rest = remove_credentials(deps.storage, idx)?;
            assert_owner_derivable(rest, owner.to_string())?;
        },        
    }
    Ok(Response::new().add_attributes(vec![("action", "update_account_data")]))
}



pub fn try_updating_ownership(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    new_owner: String,
    new_account_data: Option<CredentialData>,
) -> ContractResult {
    assert_registry(deps.storage, info.sender.as_str())?;
    let owner = get_ownership(deps.storage)?.owner.unwrap();

    ensure!(new_owner != owner.to_string(), ContractError::Generic(String::from(
        "New owner must be different from the current owner",
    )));

    saa_wasm::reset_credentials(deps.storage, false, true)?;

    if let Some(data) = new_account_data {
        STATUS.save(deps.storage, &Status { frozen: false })?;
        save_token_credentials(&mut deps, &env, info, data, new_owner.clone())?;
        cw_ownable::initialize_owner(deps.storage, deps.api, Some(new_owner.as_str()))?;
    } else {
        STATUS.save(deps.storage, &Status { frozen: true })?;
        cw_ownable::update_ownership(deps,&env.block, &owner, cw_ownable::Action::TransferOwnership { 
            new_owner: new_owner.clone(),
            expiry: None,
        })?;
    }
        
    Ok(Response::default()
        .add_attribute("action", "update_ownership")
        .add_attribute("new_owner", new_owner.as_str()))
}



pub fn try_updating_known_on_receive(
    deps: DepsMut,
    collection: String,
    token_id: String,
) -> ContractResult {
    KNOWN_TOKENS.save(
        deps.storage,
        (collection.as_str(), token_id.as_str()),
        &true,
    )?;
    Ok(Response::default().add_attributes(vec![
        ("action", "update_known_on_receive"),
        ("collection", collection.as_str()),
        ("token_id", token_id.as_str()),
    ]))
}

pub fn try_freezing(deps: DepsMut) -> ContractResult {
    let token = TOKEN_INFO.load(deps.storage)?;
    let owner = cw_ownable::get_ownership(deps.storage)?.owner.unwrap();

    // only allow freezing if the token owner is differnt from the stored owner
    ensure!(
        verify_nft_ownership(&deps.querier, owner.as_str(), token).is_err(),
        ContractError::Unauthorized(
            "Can only freeze if the owner has changed or called by the owner".into()
        )
    );
    STATUS.save(deps.storage, &Status { frozen: true })?;
    Ok(Response::default().add_attribute("action", "freeze"))
}



pub fn try_purging(api: &dyn Api, store: &mut dyn Storage, sender: &str) -> ContractResult {
    assert_registry(store, sender)?;
    cw_ownable::initialize_owner(store, api, None)?;
    saa_wasm::reset_credentials(store, true, true)?;
    SUPPORTED_INTERFACES.clear(store);
    CONTRACT.remove(store);
    REGISTRY_ADDRESS.remove(store);
    TOKEN_INFO.remove(store);
    STATUS.remove(store);
    KNOWN_TOKENS.clear(store);
    Ok(Response::default().add_attribute("action", "purge"))
}
