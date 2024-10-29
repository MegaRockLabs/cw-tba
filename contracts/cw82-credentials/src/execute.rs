use crate::{
    action::execute_action, error::ContractError, msg::{ContractResult, SignedActions}, state::{
        save_credentials, CREDENTIALS, KNOWN_TOKENS, MINT_CACHE, NONCES, REGISTRY_ADDRESS, SERIAL, STATUS, TOKEN_INFO, VERIFYING_CRED_ID, WITH_CALLER
    }, utils::{assert_ok_cosmos_msg, assert_owner_derivable, assert_registry, assert_signed_msg, assert_status}
};
use cosmwasm_std::{ensure, from_json, to_json_binary, Addr, CosmosMsg, DepsMut, Env, MessageInfo, Response};
use cw2::CONTRACT;
use cw22::SUPPORTED_INTERFACES;
use cw_ownable::{get_ownership, is_owner};
use cw_tba::{verify_nft_ownership, Status};
use saa::CredentialData;


pub fn try_executing(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msgs: Vec<CosmosMsg<SignedActions>>,
) -> ContractResult {
    assert_status(deps.storage)?;
    let mut res = Response::new();

    let with_caller = WITH_CALLER.load(deps.storage)?;
    let is_owner = is_owner(deps.storage, &info.sender)?;

    for msg in msgs {

        if let CosmosMsg::Custom(signed) = msg.clone() {
            
            assert_signed_msg(deps.as_ref(), &env, &signed)?;
            NONCES.save(deps.storage, signed.data.nonce.u128(), &true)?;

            for action in signed.data.messages {
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
            ensure!(with_caller && is_owner, ContractError::Unauthorized {});
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
    data: CredentialData,
) -> ContractResult {
    let owner = if is_owner(deps.storage, &info.sender)? {
        info.sender.clone()
    } else {
        assert_owner_derivable(deps.as_ref(), &data)?;
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
    assert_registry(deps.storage, &info.sender)?;

    if new_account_data.is_some() {
        let data = new_account_data.clone().unwrap();
        STATUS.save(deps.storage, &Status { frozen: false })?;
        save_credentials(deps.api, deps.storage, &env, info, data, new_owner.clone())?;
    } else {
        let owner = get_ownership(deps.storage)?.owner.unwrap().to_string();
        if new_owner == owner {
            CREDENTIALS.clear(deps.storage);
        } else {
            STATUS.save(deps.storage, &Status { frozen: true })?;
        }
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

    ensure!(
        verify_nft_ownership(&deps.querier, owner.as_str(), token).is_err(),
        ContractError::Unauthorized {}
    );

    STATUS.save(deps.storage, &Status { frozen: true })?;

    Ok(Response::default().add_attribute("action", "freeze"))
}


pub fn try_purging(deps: DepsMut, sender: Addr) -> ContractResult {
    assert_registry(deps.storage, &sender)?;

    cw_ownable::initialize_owner(deps.storage, deps.api, None)?;

    SUPPORTED_INTERFACES.clear(deps.storage);
    CONTRACT.remove(deps.storage);

    REGISTRY_ADDRESS.remove(deps.storage);
    TOKEN_INFO.remove(deps.storage);
    MINT_CACHE.remove(deps.storage);
    STATUS.remove(deps.storage);
    SERIAL.remove(deps.storage);
    VERIFYING_CRED_ID.remove(deps.storage);
    WITH_CALLER.remove(deps.storage);
    CREDENTIALS.clear(deps.storage);
    KNOWN_TOKENS.clear(deps.storage);

    Ok(Response::default().add_attribute("action", "purge"))
}
