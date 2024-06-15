use cosmwasm_std::{
    to_json_binary, Binary, Coin, CosmosMsg, DepsMut, Empty, Env, MessageInfo, ReplyOn, Response, StdResult, SubMsg, WasmMsg
};
use cw_tba::{query_tokens, verify_nft_ownership};
use crate::{
    msg::{ContractResult, Status}, 
    state::{KNOWN_TOKENS, MINT_CACHE, STATUS, TOKEN_INFO}, 
    utils::assert_status
};


pub const MINT_REPLY_ID: u64 = 1;


pub fn try_minting_token(
    deps        :    &mut DepsMut,
    info        :    &MessageInfo,
    collection  :    String,
    mint_msg    :    Binary,
) -> ContractResult {
    assert_status(deps.storage)?;
    MINT_CACHE.save(deps.storage, &collection)?;
    Ok(Response::new()
        .add_submessage(SubMsg {
                msg: WasmMsg::Execute { 
                    contract_addr   :  collection, 
                    msg             :  mint_msg, 
                    funds           :  info.funds.clone()
                }.into(),
                reply_on: ReplyOn::Success,
                id: MINT_REPLY_ID,
                gas_limit: None,
            }
        )
    )
}


pub fn try_freezing(
    deps    :  &mut DepsMut,
) -> ContractResult {
    STATUS.save(deps.storage, &Status { frozen: true })?;
    Ok(Response::default()
        .add_attribute("action", "freeze")
    )
}


pub fn try_unfreezing(
    deps: &mut DepsMut,
) -> ContractResult {
    let owner = cw_ownable::get_ownership(deps.storage)?.owner.unwrap();
    let token = TOKEN_INFO.load(deps.storage)?;

    verify_nft_ownership(
        &deps.querier, 
        owner.as_str(), 
        token
    )?;

    Ok(Response::default()
        .add_attribute("action", "unfreeze"
    ))
}




pub fn try_forgeting_tokens(
    deps        :  &mut DepsMut,
    collection  :  String,
    token_ids   :  Vec<String>,
) -> ContractResult {
    assert_status(deps.storage)?;
    let ids = if token_ids.len() == 0 {
        KNOWN_TOKENS
        .prefix(collection.as_str())
        .keys(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect::<StdResult<Vec<String>>>()?

    } else {
        token_ids
    };

    for id in ids {
        KNOWN_TOKENS.remove(
            deps.storage, 
            (collection.as_str(), id.as_str()), 
        );
    }

    Ok(Response::new().add_attribute("action", "forget_tokens"))
}



pub fn try_updating_known_tokens(
    deps         :  &mut DepsMut,
    env          :  &Env,
    collection   :  String,
    start_after  :  Option<String>,
    limit        :  Option<u32>
) -> ContractResult {
    assert_status(deps.storage)?;

    let res = query_tokens(
        &deps.querier, 
        &collection, 
        env.contract.address.to_string(), 
        start_after, 
        limit
    )?;

    for id in res.tokens {
        KNOWN_TOKENS.save(
            deps.storage, 
            (collection.as_str(), id.as_str()),
            &true
        )?;
    }

    Ok(Response::new()
        .add_attributes(vec![
            ("action",     "update_known_tokens"),
            ("collection", collection.as_str())
        ])
    )
}




pub fn try_transfering_token(
    deps        :   &mut DepsMut,
    collection  :   String,
    token_id    :   String,
    recipient   :   String,
    funds       :   Vec<Coin>
) -> ContractResult {
    assert_status(deps.storage)?;

    KNOWN_TOKENS.remove(
        deps.storage, 
        (collection.as_str(), token_id.as_str()),
    );

    let msg : CosmosMsg = WasmMsg::Execute { 
        contract_addr: collection, 
        msg: to_json_binary(&cw721_base::ExecuteMsg::<Empty, Empty>::TransferNft { 
            recipient, 
            token_id, 
        })?, 
        funds
    }.into();

    Ok(Response::default()
        .add_message(msg)
        .add_attribute("action", "transfer_token")
    )
}




pub fn try_sending_token(
    deps        :   &mut DepsMut,
    collection  :   String,
    token_id    :   String,
    contract    :   String,
    msg         :   Binary,
    funds       :   Vec<Coin>
) -> ContractResult {
    assert_status(deps.storage)?;
    KNOWN_TOKENS.remove(
        deps.storage, 
        (collection.as_str(), token_id.as_str()),
    );
    let msg : CosmosMsg = WasmMsg::Execute { 
        contract_addr: collection, 
        msg: to_json_binary(&cw721_base::ExecuteMsg::<Empty, Empty>::SendNft { 
            contract, 
            token_id, 
            msg
        })?, 
        funds
    }.into();

    Ok(Response::default()
        .add_message(msg)
        .add_attribute("action", "send_token")
    )
}


