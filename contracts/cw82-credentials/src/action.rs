use crate::{
    error::ContractError, msg::ContractResult, state::{KNOWN_TOKENS, MINT_CACHE, STATUS, TOKEN_INFO}, utils::assert_status
};
use cosmwasm_std::{
    to_json_binary, Binary, Coin, CosmosMsg, Empty, Env, MessageInfo, QuerierWrapper, ReplyOn, Response, StdResult, Storage, SubMsg, WasmMsg
};
use cw_tba::{encode_feegrant_msg, query_tokens, verify_nft_ownership, BasicAllowance, ExecuteAccountMsg, Status};

pub const MINT_REPLY_ID: u64 = 1;


pub fn execute_action(
    querier: &QuerierWrapper,
    storage: &mut dyn Storage,
    env: &Env,
    info: &MessageInfo,
    msg: ExecuteAccountMsg,
) -> ContractResult {
    assert_status(storage)?;

    match msg {

        ExecuteAccountMsg::Execute { msgs } => try_executing(msgs),

        ExecuteAccountMsg::MintToken {
            minter: collection,
            msg,
        } => try_minting_token(storage, info, collection, msg),

        ExecuteAccountMsg::TransferToken {
            collection,
            token_id,
            recipient,
        } => {
            try_transfering_token(storage, collection, token_id, recipient, info.funds.clone())
        }

        ExecuteAccountMsg::SendToken {
            collection,
            token_id,
            contract,
            msg,
        } => try_sending_token(
            storage,
            collection,
            token_id,
            contract,
            msg,
            info.funds.clone(),
        ),

        ExecuteAccountMsg::UpdateKnownTokens {
            collection,
            start_after,
            limit,
        } => try_updating_known_tokens(querier, storage, env, collection, start_after, limit),

        ExecuteAccountMsg::ForgetTokens {
            collection,
            token_ids,
        } => try_forgeting_tokens(storage, collection, token_ids),

        ExecuteAccountMsg::Freeze {} => try_freezing(storage),

        ExecuteAccountMsg::Unfreeze {} => try_unfreezing(querier, storage),

        ExecuteAccountMsg::FeeGrant { 
            grantee, 
            allowance 
        } => try_fee_granting(env.contract.address.as_str(), grantee.as_str(), allowance),

        _ => Err(ContractError::NotSupported {}),
    }
}


pub fn try_executing(
    msgs: Vec<CosmosMsg>,
) -> ContractResult {
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
            contract_addr: collection,
            msg: mint_msg,
            funds: info.funds.clone(),
        }
        .into(),
        reply_on: ReplyOn::Success,
        id: MINT_REPLY_ID,
        gas_limit: None,
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
    Ok(Response::default().add_attribute("action", "unfreeze"))
}


pub fn try_forgeting_tokens(
    storage: &mut dyn Storage,
    collection: String,
    token_ids: Vec<String>,
) -> ContractResult {
    let ids = if token_ids.len() == 0 {
        KNOWN_TOKENS
            .prefix(collection.as_str())
            .keys(storage, None, None, cosmwasm_std::Order::Ascending)
            .collect::<StdResult<Vec<String>>>()?
    } else {
        token_ids
    };

    for id in ids {
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

    let res = query_tokens(
        &querier,
        &collection,
        env.contract.address.to_string(),
        start_after,
        limit,
    )?;

    for id in res.tokens {
        KNOWN_TOKENS.save(storage, (collection.as_str(), id.as_str()), &true)?;
    }

    Ok(Response::new().add_attributes(vec![
        ("action", "update_known_tokens"),
        ("collection", collection.as_str()),
    ]))
}


pub fn try_transfering_token(
    storage: &mut dyn Storage,
    collection: String,
    token_id: String,
    recipient: String,
    funds: Vec<Coin>,
) -> ContractResult {

    KNOWN_TOKENS.remove(storage, (collection.as_str(), token_id.as_str()));

    let msg: CosmosMsg = WasmMsg::Execute {
        contract_addr: collection,
        msg: to_json_binary(&cw721_base::ExecuteMsg::<Empty, Empty>::TransferNft {
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
    storage: &mut dyn Storage,
    collection: String,
    token_id: String,
    contract: String,
    msg: Binary,
    funds: Vec<Coin>,
) -> ContractResult {
    KNOWN_TOKENS.remove(storage, (collection.as_str(), token_id.as_str()));
    let msg: CosmosMsg = WasmMsg::Execute {
        contract_addr: collection,
        msg: to_json_binary(&cw721_base::ExecuteMsg::<Empty, Empty>::SendNft {
            contract,
            token_id,
            msg,
        })?,
        funds,
    }
    .into();

    Ok(Response::default()
        .add_message(msg)
        .add_attribute("action", "send_token"))
}



pub fn try_fee_granting(
    contract: &str, 
    grantee: &str, 
    allowance: Option<BasicAllowance>
) -> Result<Response, ContractError> {

    let msg = encode_feegrant_msg(
        contract,
        grantee,
        allowance,
    )?;

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "fee_grant"))
}