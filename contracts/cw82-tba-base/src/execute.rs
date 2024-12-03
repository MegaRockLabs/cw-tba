use crate::{
    error::ContractError,
    msg::Status,
    state::{KNOWN_TOKENS, MINT_CACHE, PUBKEY, REGISTRY_ADDRESS, SERIAL, STATUS, TOKEN_INFO},
    utils::{assert_registry, assert_status, is_ok_cosmos_msg},
};
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, ReplyOn, Response,
    StdResult, SubMsg, WasmMsg,
};
use cw_ownable::{assert_owner, initialize_owner, is_owner};
use cw_tba::{encode_feegrant_msg, query_tokens, verify_nft_ownership, BasicAllowance, Cw721Msg};

pub const MINT_REPLY_ID: u64 = 1;

pub fn try_executing(
    deps: Deps,
    sender: Addr,
    msgs: Vec<CosmosMsg>,
) -> Result<Response, ContractError> {
    assert_owner(deps.storage, &sender)?;
    assert_status(deps.storage)?;
    if !msgs.iter().all(is_ok_cosmos_msg) {
        return Err(ContractError::NotSupported {});
    }
    Ok(Response::new().add_messages(msgs))
}

pub fn try_minting_token(
    deps: DepsMut,
    sender: Addr,
    collection: String,
    msg: Binary,
    funds: Vec<Coin>,
) -> Result<Response, ContractError> {
    assert_owner(deps.storage, &sender)?;
    assert_status(deps.storage)?;
    MINT_CACHE.save(deps.storage, &collection)?;
    Ok(Response::new().add_submessage(SubMsg {
        msg: WasmMsg::Execute {
            contract_addr: collection.clone(),
            msg,
            funds,
        }
        .into(),
        reply_on: ReplyOn::Success,
        id: MINT_REPLY_ID,
        gas_limit: None
    }))
}

pub fn try_freezing(deps: DepsMut, sender: Addr) -> Result<Response, ContractError> {
    let token = TOKEN_INFO.load(deps.storage)?;
    let owner = cw_ownable::get_ownership(deps.storage)?.owner.unwrap();
    if owner != sender {
        // check if current owner still holds the token
        let verification = verify_nft_ownership(&deps.querier, owner.as_str(), token);

        if verification.is_ok() {
            // the token is not in escrow it isn't freezable by other entities
            return Err(ContractError::Unauthorized {});
        }
    }

    STATUS.save(deps.storage, &Status { frozen: true })?;
    Ok(Response::default().add_attribute("action", "freeze"))
}

pub fn try_unfreezing(deps: DepsMut) -> Result<Response, ContractError> {
    let owner = cw_ownable::get_ownership(deps.storage)?.owner.unwrap();
    let token = TOKEN_INFO.load(deps.storage)?;
    verify_nft_ownership(&deps.querier, owner.as_str(), token)?;
    Ok(Response::default().add_attribute("action", "unfreeze"))
}

pub fn try_updating_ownership(
    deps: DepsMut,
    sender: Addr,
    new_owner: String,
    new_pubkey: Option<Binary>,
) -> Result<Response, ContractError> {
    assert_registry(deps.storage, &sender)?;
    initialize_owner(deps.storage, deps.api, Some(&new_owner))?;
    if new_pubkey.is_some() {
        PUBKEY.save(deps.storage, &new_pubkey.unwrap())?;
        STATUS.save(deps.storage, &Status { frozen: false })?;
    }
    Ok(Response::default()
        .add_attribute("action", "update_ownership")
        .add_attribute("new_owner", new_owner.as_str()))
}

pub fn try_changing_pubkey(
    deps: DepsMut,
    sender: Addr,
    pubkey: Binary,
) -> Result<Response, ContractError> {
    assert_owner(deps.storage, &sender)?;
    assert_status(deps.storage)?;
    PUBKEY.save(deps.storage, &pubkey)?;
    Ok(Response::new().add_attributes(vec![
        ("action", "change_pubkey"),
        ("new_pubkey", pubkey.to_base64().as_str()),
    ]))
}

pub fn try_forgeting_tokens(
    deps: DepsMut,
    sender: Addr,
    collection: String,
    token_ids: Vec<String>,
) -> Result<Response, ContractError> {
    assert_owner(deps.storage, &sender)?;
    assert_status(deps.storage)?;
    let ids = if token_ids.is_empty() {
        KNOWN_TOKENS
            .prefix(collection.as_str())
            .keys(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .collect::<StdResult<Vec<String>>>()?
    } else {
        token_ids
    };

    for id in ids {
        KNOWN_TOKENS.remove(deps.storage, (collection.as_str(), id.as_str()));
    }

    Ok(Response::new().add_attribute("action", "forget_tokens"))
}

pub fn try_updating_known_tokens(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    collection: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<Response, ContractError> {
    assert_status(deps.storage)?;
    if !is_owner(deps.storage, &sender)? && env.contract.address != sender {
        return Err(ContractError::Ownership(
            cw_ownable::OwnershipError::NotOwner,
        ));
    }

    let res = query_tokens(
        &deps.querier,
        &collection,
        env.contract.address.to_string(),
        start_after,
        limit,
    )?;

    for id in res.tokens {
        KNOWN_TOKENS.save(deps.storage, (collection.as_str(), id.as_str()), &true)?;
    }

    Ok(Response::new().add_attribute("action", "update_known_tokens"))
}

pub fn try_updating_known_on_receive(
    deps: DepsMut,
    collection: String,
    token_id: String,
) -> Result<Response, ContractError> {
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

pub fn try_transfering_token(
    deps: DepsMut,
    collection: String,
    token_id: String,
    recipient: String,
    funds: Vec<Coin>,
) -> Result<Response, ContractError> {
    assert_status(deps.storage)?;
    KNOWN_TOKENS.remove(deps.storage, (collection.as_str(), token_id.as_str()));
    let msg: CosmosMsg = WasmMsg::Execute {
        contract_addr: collection,
        msg: to_json_binary(&Cw721Msg::TransferNft {
            recipient,
            token_id,
        })?,
        funds,
    }
    .into();
    Ok(Response::default()
        .add_message(msg)
        .add_attribute("action", "transfer_token"))
}

pub fn try_sending_token(
    deps: DepsMut,
    collection: String,
    token_id: String,
    contract: String,
    msg: Binary,
    funds: Vec<Coin>,
) -> Result<Response, ContractError> {
    assert_status(deps.storage)?;
    KNOWN_TOKENS.remove(deps.storage, (collection.as_str(), token_id.as_str()));
    let msg: CosmosMsg = WasmMsg::Execute {
        contract_addr: collection,
        msg: to_json_binary(&Cw721Msg::SendNft {
            contract,
            token_id,
            msg: msg.to_vec().into(),
        })?,
        funds,
    }
    .into();
    Ok(Response::default()
        .add_message(msg)
        .add_attribute("action", "send_token"))
}

pub fn try_purging(deps: DepsMut, sender: Addr) -> Result<Response, ContractError> {
    assert_registry(deps.storage, &sender)?;
    KNOWN_TOKENS.clear(deps.storage);
    REGISTRY_ADDRESS.remove(deps.storage);
    TOKEN_INFO.remove(deps.storage);
    SERIAL.remove(deps.storage);
    PUBKEY.remove(deps.storage);
    STATUS.remove(deps.storage);
    Ok(Response::default().add_attribute("action", "purge"))
}


pub fn try_fee_granting(deps: DepsMut, contract: Addr, sender: Addr, grantee: String, allowance: Option<BasicAllowance>) -> Result<Response, ContractError> {
    assert_owner(deps.storage, &sender)?;
    assert_status(deps.storage)?;

    let msg = encode_feegrant_msg(
        contract.as_str(),
        &grantee,
        allowance,
    )?;

    Ok(Response::new().add_message(msg))
}