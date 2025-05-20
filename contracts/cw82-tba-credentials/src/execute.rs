use crate::{
    action::execute_action, error::ContractError, msg::ContractResult, state::{
        save_token_credentials, KNOWN_TOKENS, REGISTRY_ADDRESS, STATUS, TOKEN_INFO
    }, utils::{assert_ok_cosmos_msg, assert_owner_derivable, assert_registry, assert_status, change_cosmos_msg}
};
use cosmwasm_std::{ensure, Api, CosmosMsg, DepsMut, Env, MessageInfo, Response, Storage};
use cw2::CONTRACT;
use cw22::SUPPORTED_INTERFACES;
use cw_ownable::get_ownership;
use cw_tba::{verify_nft_ownership, ExecuteAccountMsg, Status};
use cw_auths::{ 
    add_credentials, has_natives, remove_credentials, UpdateOperation, 
    saa_types::{CredentialData, msgs::SignedDataMsg}
};



pub fn try_executing(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msgs: Vec<CosmosMsg<SignedDataMsg>>,
) -> ContractResult {
    assert_status(deps.storage)?;
    let mut res = Response::new();

    for msg in msgs {

        if let CosmosMsg::Custom(signed) = msg.clone() {

            let actions : Vec<ExecuteAccountMsg> = cw_auths::verify_signed_actions(
                deps.api,
                deps.storage, 
                &env,
                signed.clone()
            )?;

            for action in actions {
                // assert_valid_signed_action(&action)?;
                let action_res = execute_action(&deps.querier, deps.storage, &env, &info, action)?;
                res = res
                    .add_submessages(action_res.messages)
                    .add_events(action_res.events)
                    .add_attributes(action_res.attributes);
                if let Some(data) = action_res.data {
                    res = res.set_data(data);
                }
            }
        } else {
            cw_auths::verify_native( deps.storage,  info.sender.to_string())?;
            let msg = change_cosmos_msg(msg)?;
            assert_ok_cosmos_msg(&msg)?;
            res = res.add_message(msg);
        }
    }
    
    Ok(res)
}



pub fn try_updating_account_data(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    op: UpdateOperation
) -> ContractResult {
    let had_natives = has_natives(deps.storage);
    let ownership = cw_ownable::get_ownership(deps.storage)?;
    let owner = ownership.owner.unwrap();

    match op {
        UpdateOperation::Add(cred_data) => {
            let data = cred_data.with_native(&info);
            add_credentials(deps.api, deps.storage, data.clone(), had_natives)?;  
            if let Some(pending) = ownership.pending_owner {
                assert_owner_derivable(deps.api, deps.storage, &data, Some(pending.to_string()))?;
                STATUS.save(deps.storage, &Status { frozen: false })?;
                cw_ownable::update_ownership(deps, &env.block, &pending, cw_ownable::Action::AcceptOwnership)?; 
            }
        },
        UpdateOperation::Remove(idx) => {
            let rest = remove_credentials(deps.storage, idx, had_natives)?;
            let derivable_found = rest
                    .into_iter()
                    .find_map(|(id, info)| {
                        if info.hrp.is_none() {
                            return None;
                        }
                        let addr = info.cosmos_address(deps.api, id);
                        if let Ok(addr) = addr {
                            if addr == owner {
                                return Some(addr);
                            }
                        }
                        None
            });
            match derivable_found {
                Some(addr) => {
                    if addr != owner {
                        return Err(ContractError::Generic(
                            "Cannot remove credentials that derive to the current owner".into(),
                        ));
                    }
                },
                None => return Err(ContractError::NoOwnerCred {})
            }
        },        
    }
    Ok(Response::new().add_attributes(vec![("action", "update_account_data")]))
}



pub fn try_updating_ownership(
    deps: DepsMut,
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

    cw_auths::reset_credentials(deps.storage, false, true)?;

    if let Some(data) = new_account_data {
        STATUS.save(deps.storage, &Status { frozen: false })?;
        save_token_credentials(deps.api, deps.storage, &env, info, data, new_owner.clone())?;
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
    cw_auths::reset_credentials(store, true, true)?;
    SUPPORTED_INTERFACES.clear(store);
    CONTRACT.remove(store);
    REGISTRY_ADDRESS.remove(store);
    TOKEN_INFO.remove(store);
    STATUS.remove(store);
    KNOWN_TOKENS.clear(store);
    Ok(Response::default().add_attribute("action", "purge"))
}
