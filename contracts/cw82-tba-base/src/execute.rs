use crate::{
    error::ContractError,
    msg::Status,
    state::{KNOWN_TOKENS, MINT_CACHE, PUBKEY, REGISTRY_ADDRESS, SERIAL, STATUS, TOKEN_INFO},
    utils::{assert_ok_cosmos_msg, assert_registry, assert_status, change_cosmos_msg, extract_pubkey},
};
use cosmwasm_std::{
    ensure, to_json_binary, Addr, Binary, Coin, Deps, DepsMut, Env, MessageInfo, ReplyOn, Response, StdResult, SubMsg, WasmMsg
};
use cw_auths::UpdateOperation;
use cw_ownable::{assert_owner, get_ownership, is_owner, OwnershipError};
use cw_tba::{encode_feegrant_msg, query_tokens, verify_nft_ownership, BasicAllowance, CosmosMsg, Cw721Msg};
use cw_auths::saa_types::CredentialData;

pub const MINT_REPLY_ID: u64 = 1;

pub fn try_executing(
    deps: Deps,
    sender: Addr,
    msgs: Vec<CosmosMsg>,
) -> Result<Response, ContractError> {
    assert_owner(deps.storage, &sender)?;
    assert_status(deps.storage)?;

    let msgs = msgs
        .into_iter()
        .map(|msg| {
            let msg = change_cosmos_msg(msg)?;
            assert_ok_cosmos_msg(&msg)?;
            Ok(msg)

        })
        .collect::<Result<Vec<cosmwasm_std::CosmosMsg>, ContractError>>()?;

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
    env: Env,
    info: MessageInfo,
    new_owner: String,
    new_data: Option<CredentialData>,
) -> Result<Response, ContractError> {
    assert_registry(deps.storage, &info.sender)?;
    let ownership = get_ownership(deps.storage)?;
    let addr  = deps.api.addr_validate(&new_owner)?;

    if let Some(data) = new_data {
        let new_pubkey =  extract_pubkey(deps.api, data, &addr)?;
        PUBKEY.save(deps.storage, &new_pubkey)?;
        STATUS.save(deps.storage, &Status { frozen: false })?;
        cw_ownable::initialize_owner(deps.storage, deps.api, Some(new_owner.as_str()))?;
    } else {
        STATUS.save(deps.storage, &Status { frozen: true })?;
        cw_ownable::update_ownership(deps, &env.block, &ownership.owner.unwrap(), cw_ownable::Action::TransferOwnership {
            new_owner: new_owner.to_string(),
            expiry: None,
        })?;
    }

    Ok(Response::default()
        .add_attribute("action", "update_ownership")
        .add_attribute("new_owner", new_owner.as_str()))
}

pub fn try_changing_data(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    op: UpdateOperation,
) -> Result<Response, ContractError> {
     match op {
        UpdateOperation::Add(data) => {
            let ownershop = get_ownership(deps.storage)?;
            let owner = ownershop.owner.unwrap();
            let new_owner = if let Some(pending) = ownershop.pending_owner {
                ensure!(pending == info.sender, ContractError::Unauthorized {});
                STATUS.save(deps.storage, &Status { frozen: false })?;
                true
            } else {
                ensure!(owner == info.sender, OwnershipError::NotOwner {});
                false
            };
            let pubkey = extract_pubkey(deps.api, data, &info.sender)?;
            PUBKEY.save(deps.storage, &pubkey)?;
            if new_owner {
                cw_ownable::update_ownership(deps, &env.block, &info.sender, cw_ownable::Action::AcceptOwnership)?;
                /* cw_ownable::update_ownership(deps, &env.block, &info.sender, cw_ownable::Action::TransferOwnership {
                    new_owner: info.sender.to_string(),
                    expiry: None,
                })?; */
            }
            Ok(Response::new()
                .add_attribute("action", "change_pubkey")
                .add_attribute("new_owner", info.sender.as_str()))
        },
        UpdateOperation::Remove(_) => Err(ContractError::NotSupported {}),
    }
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
    let msg: cosmwasm_std::CosmosMsg = WasmMsg::Execute {
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
    let msg: cosmwasm_std::CosmosMsg = WasmMsg::Execute {
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