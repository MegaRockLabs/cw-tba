use cosmwasm_std::{
    ensure, Addr, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response
};
use cw_ownable::{get_ownership, is_owner};
use cw_tba::verify_nft_ownership;
use saa::CredentialData;
use crate::{
    error::ContractError, 
    msg::{ContractResult, SignedCosmosMsgs, Status}, 
    state::{save_credentials, CREDENTIALS, KNOWN_TOKENS, MINT_CACHE, REGISTRY_ADDRESS, SECS_TO_EXPIRE, SERIAL, STATUS, TOKEN_INFO, VERIFYING_CRED_ID, WITH_CALLER}, 
    utils::{checked_execute_msgs, assert_owner_derivable, assert_registry, assert_status}
};

pub const MINT_REPLY_ID: u64 = 1;


pub fn try_executing(
    deps    :   Deps,
    env     :   Env,
    info    :   MessageInfo,
    msgs    :   Vec<CosmosMsg<SignedCosmosMsgs>>
) -> ContractResult {
    assert_status(deps.storage)?;
    let msgs = checked_execute_msgs(deps, &env, info.sender.as_str(), &msgs)?;
    Ok(Response::new().add_messages(msgs))
}



pub fn try_updating_account_data(
    deps     :   DepsMut,
    env      :   Env,
    info     :   MessageInfo,
    data     :   CredentialData
) -> ContractResult {

    let owner = if is_owner(deps.storage, &info.sender)? {
        info.sender.clone()
    } else {
        assert_owner_derivable(deps.as_ref(), &data)?;
        get_ownership(deps.storage)?.owner.unwrap()
    };

    save_credentials(deps, env, info, data, owner.to_string())?;

    Ok(Response::new()
        .add_attributes(vec![
            ("action", "update_account_data"),
        ])
    )
}



pub fn try_updating_ownership(
    deps              :   DepsMut,
    env               :   Env,
    info              :   MessageInfo,
    new_owner         :   String,
    new_account_data  :   Option<CredentialData>,
) -> ContractResult {
    assert_registry(deps.storage, &info.sender)?;

    if new_account_data.is_some() {
        let data = new_account_data.clone().unwrap();
        STATUS.save(deps.storage, &Status { frozen: false })?;
        save_credentials(deps, env, info, data, new_owner.clone())?;
    } else {
        STATUS.save(deps.storage, &Status { frozen: true })?;
    }

    Ok(Response::default()
        .add_attribute("action", "update_ownership")
        .add_attribute("new_owner", new_owner.as_str())
    )
}


pub fn try_updating_known_on_receive(
    deps: DepsMut,
    collection: String,
    token_id: String,
) -> ContractResult {
    KNOWN_TOKENS.save(
        deps.storage, 
        (collection.as_str(), token_id.as_str()),
        &true
    )?;
    Ok(Response::default()
        .add_attributes(vec![
            ("action", "update_known_on_receive"),
            ("collection", collection.as_str()),
            ("token_id", token_id.as_str())
        ])
    )
}




pub fn try_freezing(
    deps    :  DepsMut,
) -> ContractResult {
    let token =   TOKEN_INFO.load(deps.storage)?;
    let owner      =   cw_ownable::get_ownership(deps.storage)?.owner.unwrap();

    ensure!(
        verify_nft_ownership(&deps.querier, owner.as_str(), token).is_err(),
        ContractError::Unauthorized {}
    );

    STATUS.save(deps.storage, &Status { frozen: true })?;

    Ok(Response::default()
        .add_attribute("action", "freeze")
    )
}



pub fn try_purging(
    deps: DepsMut,
    sender: Addr
) -> ContractResult {
    assert_registry(deps.storage, &sender)?;

    REGISTRY_ADDRESS.remove(deps.storage);
    TOKEN_INFO.remove(deps.storage);
    MINT_CACHE.remove(deps.storage);
    STATUS.remove(deps.storage);
    SERIAL.remove(deps.storage);
    VERIFYING_CRED_ID.remove(deps.storage);
    WITH_CALLER.remove(deps.storage);
    SECS_TO_EXPIRE.remove(deps.storage);
    CREDENTIALS.clear(deps.storage);
    KNOWN_TOKENS.clear(deps.storage);
    
    Ok(Response::default()
        .add_attribute("action", "purge")
    )
}
