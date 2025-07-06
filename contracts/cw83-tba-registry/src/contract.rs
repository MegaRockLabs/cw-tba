#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::{
    error::ContractError,
    execute::{create_account, migrate_account, update_account_data, update_account_owner},
    msg::{AccountsQueryMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    query::{account_info, accounts, collection_accounts, collections},
    state::{COL_TOKEN_COUNTS, LAST_ATTEMPTING, REGISTRY_PARAMS, TOKEN_ADDRESSES},
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
    cw2::get_contract_version(deps.storage)?;
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
            create.account_data.token_info,
            create.account_data.credential_data,
            create.account_data.create_for,
            create.account_data.actions,
            false,
        ),

        ExecuteMsg::ResetAccount(create) => create_account(
            deps,
            env,
            info,
            create.chain_id,
            create.code_id,
            create.account_data.token_info,
            create.account_data.credential_data,
            create.account_data.create_for,
            create.account_data.actions,
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
        } => update_account_owner(deps, env, info, token_info, new_account_data, update_for),

        ExecuteMsg::UpdateAccountData {
            token_info,
            update_op,
            credential,
        } => update_account_data(deps, env, info, token_info, update_op, credential),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _: Env, msg: cosmwasm_std::Reply) -> Result<Response, ContractError> {
    if msg.id != cw83::CREATE_ACCOUNT_REPLY_ID {
        return Err(ContractError::Unauthorized {});
    }

    let res = cw_utils::parse_reply_instantiate_data(msg)?;
/* 
    cw22::query_supported_interface_version(
        &deps.querier,
        res.contract_address.as_str(),
        cw82::INTERFACE_NAME,
    )?; */


    let stored = LAST_ATTEMPTING.load(deps.storage)?;
    LAST_ATTEMPTING.remove(deps.storage);

    COL_TOKEN_COUNTS.update(
        deps.storage,
        stored.collection.as_str(),
        |count| -> StdResult<u32> {
            match count {
                Some(c) => Ok(c + 1),
                None => Ok(1),
            }
        },
    )?;

    TOKEN_ADDRESSES.save(deps.storage, stored.key(), &res.contract_address)?;

    Ok(Response::default())
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: cosmwasm_std::Deps, _: Env, msg: QueryMsg) -> StdResult<cosmwasm_std::Binary> {
    match msg {
        QueryMsg::AccountInfo(acc_query) => to_json_binary(&account_info(deps, acc_query)?),
        QueryMsg::Accounts {
            skip, limit, query, ..
        } => {
            let res = if let Some(q) = query {
                match q {
                    AccountsQueryMsg::Collections {} => collections(deps, skip, limit),
                    AccountsQueryMsg::Collection(col) => {
                        collection_accounts(deps, col, skip, limit)
                    }
                }
            } else {
                accounts(deps, skip, limit)
            }?;
            to_json_binary(&res)
        }

        QueryMsg::RegistryParams {} => to_json_binary(&REGISTRY_PARAMS.load(deps.storage)?),
    }
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_: DepsMut, _: Env, _: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default().add_attribute("action", "migrate"))
}
