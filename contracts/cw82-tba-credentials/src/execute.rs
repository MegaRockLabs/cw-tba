use crate::{
    action::execute_action, error::ContractError, msg::{ContractResult, SignedMessages}, state::{
        save_credentials, KNOWN_TOKENS, MINT_CACHE, REGISTRY_ADDRESS, SERIAL, STATUS, TOKEN_INFO, WITH_CALLER
    }, utils::{assert_caller, assert_ok_cosmos_msg, assert_owner_derivable, assert_registry, assert_status, assert_valid_signed_action}
};
use cosmwasm_std::{ensure, from_json, Api, CosmosMsg, DepsMut, Env, MessageInfo, Response, Storage};
use cw2::CONTRACT;
use cw22::SUPPORTED_INTERFACES;
use cw_ownable::{get_ownership, is_owner};
use cw_tba::{verify_nft_ownership, Status};
use saa::{ 
    messages::SignedDataMsg, to_json_binary, CredentialData, UpdateOperation
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

            saa::verify_signed_actions(
                deps.api,
                deps.storage, 
                &env,
                signed.clone()
            )?;

            let data : SignedMessages = from_json(&signed.data)?;

            for action in data.messages {
                assert_valid_signed_action(&action)?;
                let action_res = execute_action(&deps.querier, deps.storage, &env, &info, action)?;
                res = res
                    .add_submessages(action_res.messages)
                    .add_events(action_res.events)
                    .add_attributes(action_res.attributes);
                if action_res.data.is_some() {
                    res = res.set_data(action_res.data.unwrap());
                }
            }
        } else {
            assert_caller(deps.as_ref(), info.sender.as_str())?;
            let msg = from_json(to_json_binary(&msg)?)?;
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
    data: Option<CredentialData>,
    op: UpdateOperation
) -> ContractResult {
    ensure!(data.is_some(), ContractError::Generic(String::from("No proving data provided")));
    let data = data.unwrap();

    let owner = if is_owner(deps.storage, &info.sender)? {
        info.sender.clone()
    } else {
        data.update(op, deps.api, deps.storage, &env, &info)?;
        assert_owner_derivable(deps.api, deps.storage, &data, None)?;
        get_ownership(deps.storage)?.owner.unwrap()
    };
    save_credentials(deps.api, deps.storage, &env, info, data, owner.to_string())?;
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

    let owner = get_ownership(deps.storage)?.owner.unwrap().to_string();
    if new_owner != owner {
        saa::reset_credentials(deps.storage)?;
    }

    if new_account_data.is_some() {
        let data = new_account_data.clone().unwrap();
        save_credentials(deps.api, deps.storage, &env, info, data, new_owner.clone())?;
        STATUS.save(deps.storage, &Status { frozen: false })?;
    } else {
        cw_ownable::initialize_owner(deps.storage, deps.api, Some(new_owner.as_str()))?;
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
    saa::reset_credentials(store)?;
    SUPPORTED_INTERFACES.clear(store);
    CONTRACT.remove(store);
    REGISTRY_ADDRESS.remove(store);
    TOKEN_INFO.remove(store);
    MINT_CACHE.remove(store);
    STATUS.remove(store);
    SERIAL.remove(store);
    WITH_CALLER.remove(store);
    KNOWN_TOKENS.clear(store);
    Ok(Response::default().add_attribute("action", "purge"))
}
