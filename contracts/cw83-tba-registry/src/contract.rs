#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_json, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult, SubMsgResult
};

use cw83::CREATE_ACCOUNT_REPLY_ID;
use cw_tba::TokenInfo;

use crate::{
    error::ContractError,
    execute::{create_account, migrate_account, update_account_owner},
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    query::{account_info, accounts, collection_accounts, collections},
    state::{COL_TOKEN_COUNTS, REGISTRY_PARAMS, TOKEN_ADDRESSES},
};

pub const CONTRACT_NAME: &str = "crates:cw83-token-account-registry";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _: Env,
    _: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw22::set_contract_supported_interface(
        deps.storage,
        &[cw22::ContractSupportedInterface {
            supported_interface: cw83::INTERFACE_NAME.into(),
            version: CONTRACT_VERSION.into(),
        }],
    )?;
    REGISTRY_PARAMS.save(deps.storage, &msg.params)?;
    Ok(Response::new().add_attributes(vec![("action", "instantiate")]))
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateAccount(create) => create_account(
            deps,
            env,
            info,
            create.chain_id,
            create.code_id,
            create.msg.token_info,
            create.msg.account_data,
            create.msg.create_for,
            create.msg.actions,
            false,
        ),

        ExecuteMsg::ResetAccount(create) => create_account(
            deps,
            env,
            info,
            create.chain_id,
            create.code_id,
            create.msg.token_info,
            create.msg.account_data,
            create.msg.create_for,
            create.msg.actions,
            true,
        ),

        ExecuteMsg::MigrateAccount {
            token_info,
            new_code_id,
            msg,
        } => migrate_account(deps, info.sender, token_info, new_code_id, msg),

        ExecuteMsg::UpdateAccountOwnership {
            token_info,
            new_account_data,
            update_for,
        } => update_account_owner(
            deps,
            info.sender,
            token_info,
            new_account_data,
            info.funds,
            update_for,
        ),
    }
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _: Env, reply: Reply) -> Result<Response, ContractError> {
    if reply.id == CREATE_ACCOUNT_REPLY_ID {

        let data = if let SubMsgResult::Ok(res) = reply.result {
            res.msg_responses[0].clone()
        } else {
            return Err(ContractError::Unauthorized {})
        };
        
        let init_res = cw_utils::parse_instantiate_response_data(data.value.as_slice())?;

        let addr = init_res.contract_address;
        deps.api.addr_validate(addr.as_str())?;

        let token_info : TokenInfo = from_json(&reply.payload)?;

        COL_TOKEN_COUNTS.update(
            deps.storage,
            token_info.collection.as_str(),
            |count| -> StdResult<u32> {
                match count {
                    Some(c) => Ok(c + 1),
                    None => Ok(1),
                }
            },
        )?;

        TOKEN_ADDRESSES.save(
            deps.storage,
            (token_info.collection.as_str(), token_info.id.as_str()),
            &addr,
        )?;

        Ok(Response::default())
    } else {
        Err(ContractError::Unauthorized {})
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::AccountInfo(acc_query) => to_json_binary(&account_info(deps, acc_query.query)?),

        QueryMsg::Collections { skip, limit } => to_json_binary(&collections(deps, skip, limit)?),

        QueryMsg::Accounts { skip, limit } => to_json_binary(&accounts(deps, skip, limit)?),

        QueryMsg::CollectionAccounts {
            collection,
            skip,
            limit,
        } => to_json_binary(&collection_accounts(deps, &collection, skip, limit)?),

        QueryMsg::RegistryParams {} => to_json_binary(&REGISTRY_PARAMS.load(deps.storage)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_: DepsMut, _: Env, _: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default().add_attribute("action", "migrate"))
}
