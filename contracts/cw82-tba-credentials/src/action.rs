use crate::{
    error::ContractError,
    msg::ContractResult,
    state::{KNOWN_TOKENS, MINT_CACHE, STATUS, TOKEN_INFO},
    utils::{assert_ok_cosmos_msg, assert_status},
};
use cosmwasm_std::{
    to_json_binary, Binary, CosmosMsg, DepsMut, Env, MessageInfo, QuerierWrapper, ReplyOn, Response, StdResult, Storage, SubMsg, WasmMsg
};
use cw_tba::{
    encode_feegrant_msg, query_tokens, verify_nft_ownership, BasicAllowance, Cw721Msg,
    ExecuteAccountMsg, Status,
};

pub const MINT_REPLY_ID: u64 = 1;

pub fn execute_action(
    deps: &mut DepsMut,
    env: &Env,
    info: &MessageInfo,
    msg: ExecuteAccountMsg,
) -> ContractResult {
    assert_status(deps.storage)?;
    use ExecuteAccountMsg::*;

    match msg {
        Execute { msgs } => try_executing(msgs),

        MintToken { minter, msg } => try_minting_token(deps.storage, info, minter, msg),

        TransferToken {
            collection,
            token_id,
            recipient,
        } => try_transfering_token(deps.storage, collection, token_id, recipient),

        #[cfg(not(feature = "omniflix"))]
        SendToken {
            collection,
            token_id,
            contract,
            msg,
        } => try_sending_token(deps.storage, collection, token_id, contract, msg),

        UpdateKnownTokens {
            collection,
            start_after,
            limit,
        } => try_updating_known_tokens(
            &deps.querier,
            deps.storage,
            env,
            collection,
            start_after,
            limit,
        ),

        ForgetTokens {
            collection,
            token_ids,
        } => try_forgeting_tokens(deps.storage, collection, token_ids),

        Freeze {} => try_freezing(deps.storage),

        Unfreeze {} => try_unfreezing(&deps.querier, deps.storage),

        FeeGrant { grantee, allowance } => {
            try_fee_granting(env.contract.address.as_str(), grantee.as_str(), allowance)
        }
    }
}

pub fn try_executing(msgs: Vec<cosmwasm_std::CosmosMsg>) -> ContractResult {
    msgs.iter().try_for_each(|msg| assert_ok_cosmos_msg(msg))?;
    Ok(Response::new().add_messages(msgs))
}

pub fn try_minting_token(
    storage: &mut dyn Storage,
    info: &MessageInfo,
    collection: String,
    mint_msg: Binary,
) -> ContractResult {
    MINT_CACHE.save(storage, &collection)?;
    Ok(Response::new().add_submessage(SubMsg {
        msg: WasmMsg::Execute {
            contract_addr: collection.clone(),
            msg: mint_msg,
            funds: info.funds.clone(),
        }
        .into(),
        reply_on: ReplyOn::Success,
        id: MINT_REPLY_ID,
        gas_limit: None,
        // payload: Binary::default(),
    }))
}

pub fn try_freezing(storage: &mut dyn Storage) -> ContractResult {
    STATUS.save(storage, &Status { frozen: true })?;
    Ok(Response::default().add_attribute("action", "freeze"))
}

pub fn try_unfreezing(querier: &QuerierWrapper, storage: &mut dyn Storage) -> ContractResult {
    let owner = cw_ownable::get_ownership(storage)?.owner.unwrap();
    let token = TOKEN_INFO.load(storage)?;
    verify_nft_ownership(&querier, owner.as_str(), token)?;
    STATUS.save(storage, &Status { frozen: false })?;
    Ok(Response::default().add_attribute("action", "unfreeze"))
}

pub fn try_forgeting_tokens(
    storage: &mut dyn Storage,
    collection: String,
    token_ids: Vec<String>,
) -> ContractResult {
    for id in if token_ids.len() == 0 {
        KNOWN_TOKENS
        .prefix(collection.as_str())
        .keys(storage, None, None, cosmwasm_std::Order::Ascending)
        .collect::<StdResult<Vec<String>>>()?
    } else { token_ids } 
{
        KNOWN_TOKENS.remove(storage, (collection.as_str(), id.as_str()));
    }
    Ok(Response::new().add_attribute("action", "forget_tokens"))
}

pub fn try_updating_known_tokens(
    querier: &QuerierWrapper,
    storage: &mut dyn Storage,
    env: &Env,
    collection: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> ContractResult {
    for id in  query_tokens(
        &querier,
        &collection,
        env.contract.address.to_string(),
        start_after,
        limit,
    )?.tokens {
        KNOWN_TOKENS.save(storage, (collection.as_str(), id.as_str()), &true)?;
    }
    Ok(Response::new().add_attributes(vec![
        ("action", "update_known_tokens"),
        ("collection", collection.as_str()),
    ]))
}

pub fn try_transfering_token(
    storage: &mut dyn Storage,
    contract_addr: String,
    token_id: String,
    recipient: String,
) -> ContractResult {
    KNOWN_TOKENS.remove(storage, (contract_addr.as_str(), token_id.as_str()));
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        msg: to_json_binary(&Cw721Msg::TransferNft {
            recipient,
            token_id,
        })?,
        contract_addr,
        funds: vec![],
    });
    Ok(Response::default()
        .add_message(msg)
        .add_attribute("action", "transfer_token"))
}

#[cfg(not(feature = "omniflix"))]
pub fn try_sending_token(
    storage: &mut dyn Storage,
    contract_addr: String,
    token_id: String,
    contract: String,
    msg: Binary,
) -> ContractResult {
    KNOWN_TOKENS.remove(storage, (contract_addr.as_str(), token_id.as_str()));
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        msg: to_json_binary(&Cw721Msg::SendNft {
            contract,
            token_id,
            msg
        })?,
        contract_addr,
        funds: vec![],
    });
    Ok(Response::default()
        .add_message(msg)
        .add_attribute("action", "send_token"))
}

pub fn try_fee_granting(
    contract: &str,
    grantee: &str,
    allowance: Option<BasicAllowance>,
) -> Result<Response, ContractError> {
    let msg = encode_feegrant_msg(contract, grantee, allowance)?;

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "fee_grant"))
}
