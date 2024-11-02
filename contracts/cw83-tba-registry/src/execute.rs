use cosmwasm_schema::serde::Serialize;
use cosmwasm_std::{
    ensure, ensure_eq, to_json_binary, Addr, Binary, Coin, CosmosMsg, DepsMut, Empty, Env, MessageInfo, ReplyOn, Response, SubMsg, WasmMsg
};

use cw83::CREATE_ACCOUNT_REPLY_ID;
use cw_tba::{
    verify_nft_ownership, ExecuteAccountMsg, InstantiateAccountMsg, MigrateAccountMsg, TokenInfo,
};

type AccountMsg = ExecuteAccountMsg<Binary>;

use crate::{
    error::ContractError, funds::checked_funds, registry::construct_label, state::{LAST_ATTEMPTING, REGISTRY_PARAMS, TOKEN_ADDRESSES}
};

pub fn create_account<T: Serialize, A: Serialize>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    chain_id: String,
    code_id: u64,
    token_info: TokenInfo,
    account_data: T,
    create_for: Option<String>,
    actions: Option<Vec<A>>,
    reset: bool,
) -> Result<Response, ContractError> {
    ensure_eq!(env.block.chain_id, chain_id, ContractError::InvalidChainId {});
    
    let params = REGISTRY_PARAMS.load(deps.storage)?;
    ensure!(params.allowed_code_ids.contains(&code_id), ContractError::InvalidCodeId {});

    let sender = info.sender.to_string();
    let is_manager = params.managers.contains(&sender);
    let owner = create_for.unwrap_or(sender);

    ensure!(owner == info.sender || is_manager, ContractError::Unauthorized {});
    verify_nft_ownership(&deps.querier, owner.as_str(), token_info.clone())?;

    LAST_ATTEMPTING.save(deps.storage, &token_info)?;

    let mut msgs : Vec<CosmosMsg> = Vec::with_capacity(1);
    let funds = checked_funds(deps.storage, &info)?;

    let token_address = TOKEN_ADDRESSES.may_load(
        deps.storage,
        (token_info.collection.as_str(), token_info.id.as_str()),
    )?;

    let label = if token_address.is_some() {
        ensure!(reset, ContractError::AccountExists {});
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token_address.unwrap(),
            msg: to_json_binary(&AccountMsg::Purge {})?,
            funds: vec![],
        }));
        construct_label(&token_info, Some(env.block.height))
    } else {
        construct_label(&token_info, None)
    };


    let init_msg = InstantiateAccountMsg::<T, A> {
        owner: info.sender.to_string(),
        token_info: token_info.clone(),
        account_data,
        actions,
    };
    

    Ok(Response::default()
        .add_messages(msgs)
        .add_submessage(SubMsg {
            id: CREATE_ACCOUNT_REPLY_ID,
            msg: cosmwasm_std::CosmosMsg::Wasm(WasmMsg::Instantiate {
                admin: Some(env.contract.address.to_string()),
                msg: to_json_binary(&init_msg)?,
                code_id,
                label,
                funds,
            }),
            reply_on: ReplyOn::Success,
            gas_limit: None,
        })
        .add_attributes(vec![
            ("action", if reset { "reset_account"} else { "create_account" }),
            ("token_contract", token_info.collection.as_str()),
            ("token_id", token_info.id.as_str()),
            ("code_id", code_id.to_string().as_str()),
            ("chain_id", chain_id.as_str()),
            ("owner", info.sender.as_str()),
        ])
    )
}


pub fn update_account_owner(
    deps: DepsMut,
    sender: Addr,
    token_info: TokenInfo,
    new_account_data: Option<Binary>,
    funds: Vec<Coin>,
    update_for: Option<String>,
) -> Result<Response, ContractError> {
    let is_manager = REGISTRY_PARAMS
        .load(deps.storage)?
        .managers
        .contains(&sender.to_string());
    let owner = update_for.unwrap_or(sender.to_string());
    // only admin can update ownership but only if the new address is the token owner
    if owner != sender && !is_manager {
        return Err(ContractError::Unauthorized {});
    }
    verify_nft_ownership(&deps.querier, owner.as_str(), token_info.clone())?;
    let contract_addr = TOKEN_ADDRESSES.load(
        deps.storage,
        (token_info.collection.as_str(), token_info.id.as_str()),
    )?;
    let msg = ExecuteAccountMsg::<Empty>::UpdateOwnership {
        new_owner: owner.to_string(),
        new_account_data,
    };
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr,
        msg: to_json_binary(&msg)?,
        funds,
    });
    Ok(Response::default().add_message(msg).add_attributes(vec![
        ("action", "update_account_owner"),
        ("token_contract", token_info.collection.as_str()),
        ("token_id", token_info.id.as_str()),
        ("new_owner", owner.to_string().as_str()),
    ]))
}


pub fn migrate_account(
    deps: DepsMut,
    sender: Addr,
    token_info: TokenInfo,
    new_code_id: u64,
    msg: MigrateAccountMsg,
) -> Result<Response, ContractError> {
    if !REGISTRY_PARAMS
        .load(deps.storage)?
        .allowed_code_ids
        .contains(&new_code_id)
    {
        return Err(ContractError::InvalidCodeId {});
    }
    verify_nft_ownership(&deps.querier, sender.as_str(), token_info.clone())?;
    let contract_addr = TOKEN_ADDRESSES.load(
        deps.storage,
        (token_info.collection.as_str(), token_info.id.as_str()),
    )?;
    let msg = CosmosMsg::Wasm(WasmMsg::Migrate {
        contract_addr,
        new_code_id,
        msg: to_json_binary(&msg)?,
    });
    Ok(Response::default().add_message(msg).add_attributes(vec![
        ("action", "migrate_account"),
        ("token_contract", token_info.collection.as_str()),
        ("token_id", token_info.id.as_str()),
        ("new_code_id", new_code_id.to_string().as_str()),
    ]))
}
