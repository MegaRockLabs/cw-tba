use cosmwasm_std::{
    ensure, ensure_eq, to_json_binary, Addr, CosmosMsg, DepsMut, Env, MessageInfo, ReplyOn, Response, StdError, SubMsg, WasmMsg
};

use crate::{
    error::ContractError, funds::checked_funds, msg::SudoMsg, state::{LAST_ATTEMPTING, REGISTRY_PARAMS, TOKEN_ADDRESSES}
};
use cw83::CREATE_ACCOUNT_REPLY_ID;
use cw84::{Binary, ValidSignatureResponse};
use cw_tba::{
    verify_nft_ownership, ExecuteAccountMsg, ExecuteMsg, InstantiateAccountMsg, QueryMsg, RegistryParams, TokenInfo
};
use saa_wasm::{
    saa_types::{
        CheckOption, Credential, CredentialData, CredentialsWrapper, ReplayParams, VerifiedData
    },
    UpdateOperation,
};

const CREATE_MSG: &str = "Create TBA account";
const UPDATE_MSG: &str = "Update TBA account ownership";


fn construct_label(info: &TokenInfo, serial: Option<u64>) -> String {
    let base = format!("{}-{}-account", info.collection, info.id);
    match serial {
        Some(s) => format!("{}-{}", base, s),
        None => base,
    }
}

pub fn create_account(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    chain_id: String,
    code_id: u64,
    token_info: TokenInfo,
    account_data: CredentialData,
    create_for: Option<String>,
    actions: Option<Vec<ExecuteAccountMsg>>,
    reset: bool,
) -> Result<Response, ContractError> {
    ensure_eq!(
        env.block.chain_id,
        chain_id,
        ContractError::InvalidChainId {}
    );

    let params = REGISTRY_PARAMS.load(deps.storage)?;
    ensure!(
        params.allowed_code_ids.contains(&code_id),
        ContractError::InvalidCodeId {}
    );

    let sender = info.sender.to_string();
    let is_manager = params.managers.contains(&sender);
    let owner = create_for.unwrap_or(sender);

    ensure!(
        owner == info.sender.to_string() || is_manager,
        ContractError::Unauthorized {}
    );
    verify_nft_ownership(&deps.querier, owner.as_str(), token_info.clone())?;

    LAST_ATTEMPTING.save(deps.storage, &token_info)?;

    let mut msgs: Vec<CosmosMsg> = Vec::with_capacity(1);
    let funds = checked_funds(deps.storage, &info)?;

    let token_address = TOKEN_ADDRESSES.may_load(deps.storage, token_info.key())?;

    let label = if token_address.is_some() {
        ensure!(reset, ContractError::AccountExists {});
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token_address.unwrap(),
            msg: to_json_binary(&ExecuteMsg::Purge {})?,
            funds: vec![],
        }));
        construct_label(&token_info, Some(env.block.height))
    } else {
        construct_label(&token_info, None)
    };

    let replay_params = ReplayParams::new(0, CheckOption::Messages(vec![CREATE_MSG.into()]));

    let account_data = account_data.verify(deps.as_ref(), &env, &info, replay_params)?;

    let init_msg = InstantiateAccountMsg {
        owner: info.sender.to_string(),
        token_info: token_info.clone(),
        account_data,
        actions,
    };
    
    let action = if reset { "reset_account" } else { "create_account"};

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
            // payload: Binary::default(),
        })
        .add_attributes(vec![
            ("action", action),
            ("collection", token_info.collection.as_str()),
            ("token_id", token_info.id.as_str()),
            ("owner", info.sender.as_str()),
            ("code_id", code_id.to_string().as_str()),
            ("chain_id", chain_id.as_str()),
        ]))
}


pub fn update_account_owner(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_info: TokenInfo,
    new_account_data: Option<CredentialData>,
    update_for: Option<String>,
) -> Result<Response, ContractError> {
    let is_manager = REGISTRY_PARAMS
        .load(deps.storage)?
        .managers
        .contains(&info.sender.to_string());

    let owner = update_for.unwrap_or(info.sender.to_string());
    // only admin can update ownership but only if the new address is the token owner

    if owner != info.sender.to_string() && !is_manager {
        return Err(ContractError::Unauthorized {});
    }

    verify_nft_ownership(&deps.querier, owner.as_str(), token_info.clone())?;

    let contract_addr = TOKEN_ADDRESSES.load(deps.storage, token_info.key())?;

    ensure!(
        new_account_data.is_some() || owner != info.sender.to_string(),
        ContractError::Generic(String::from(
            "New owner must be different from the current owner",
        ))
    );

    let nonce = deps
        .querier
        .query_wasm_smart::<u64>(contract_addr.clone(), &QueryMsg::AccountNumber {})?;

    let params = ReplayParams {
        override_address: Some(contract_addr.clone()),
        ..ReplayParams::new(nonce, CheckOption::Text(UPDATE_MSG.to_string()))
    };

    let new_account_data = new_account_data
        .map(|data| data.verify(deps.as_ref(), &env, &info, params))
        .transpose()?;

    let msg = ExecuteMsg::UpdateOwnership {
        new_owner: owner.to_string(),
        new_account_data,
    };

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr,
        msg: to_json_binary(&msg)?,
        funds: info.funds,
    });

    Ok(Response::default().add_message(msg).add_attributes(vec![
        ("action", "update_account_owner"),
        ("token_contract", token_info.collection.as_str()),
        ("token_id", token_info.id.as_str()),
        ("new_owner", owner.to_string().as_str()),
    ]))
}

pub fn update_account_data(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_info: TokenInfo,
    op: UpdateOperation,
    cred: Option<Credential>,
) -> Result<Response, ContractError> {
    let contract_addr = TOKEN_ADDRESSES.load(deps.storage, token_info.key())?;

    match cred {
        Some(cred) => {
            let query = QueryMsg::ValidSignature {
                data: Binary::default(),
                signature: Binary::default(),
                payload: Some(cred),
            };
            let res: ValidSignatureResponse = deps
                .querier
                .query_wasm_smart(contract_addr.clone(), &query)?;
            ensure!(res.is_valid, ContractError::Unauthorized {});
        }
        None => {
            verify_nft_ownership(&deps.querier, info.sender.as_str(), token_info.clone())?;
        }
    }

    let op: UpdateOperation<VerifiedData> = match op {
        UpdateOperation::Remove(ids) => UpdateOperation::<VerifiedData>::Remove(ids),
        UpdateOperation::Add(data) => {
            let n = deps
                .querier
                .query_wasm_smart::<u64>(contract_addr.clone(), &QueryMsg::AccountNumber {})?;

            let params = ReplayParams {
                override_address: Some(contract_addr.clone()),
                ..ReplayParams::new(n, CheckOption::Text(UPDATE_MSG.to_string()))
            };

            let data = data.verify(deps.as_ref(), &env, &info, params)?;
            UpdateOperation::Add(data)
        }
    };

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr,
        msg: to_json_binary(&ExecuteMsg::UpdateAccountData(op))?,
        funds: info.funds,
    });

    Ok(Response::default().add_message(msg).add_attributes(vec![
        ("action", "update_account_data"),
        ("token_contract", token_info.collection.as_str()),
        ("token_id", token_info.id.as_str()),
    ]))
}


pub fn migrate_account(
    deps: DepsMut,
    sender: Addr,
    token_info: TokenInfo,
    new_code_id: u64,
    msg: Binary,
) -> Result<Response, ContractError> {
    if !REGISTRY_PARAMS
        .load(deps.storage)?
        .allowed_code_ids
        .contains(&new_code_id)
    {
        return Err(ContractError::InvalidCodeId {});
    }
    verify_nft_ownership(&deps.querier, sender.as_str(), token_info.clone())?;
    let contract_addr = TOKEN_ADDRESSES.load(deps.storage, token_info.key())?;
    let msg = CosmosMsg::Wasm(WasmMsg::Migrate {
        contract_addr,
        new_code_id,
        msg
    });
    Ok(Response::default().add_message(msg).add_attributes(vec![
        ("action", "migrate_account"),
        ("token_contract", token_info.collection.as_str()),
        ("token_id", token_info.id.as_str()),
        ("new_code_id", new_code_id.to_string().as_str()),
    ]))
}



pub fn execute_admin(
    deps: DepsMut,
    msg: SudoMsg,
) -> Result<Response, ContractError> {

    match msg {
        SudoMsg::UpdateAllowedCodeIds { code_ids } => {
            REGISTRY_PARAMS.update(deps.storage, |mut params| {
                params.allowed_code_ids = code_ids;
                Ok::<RegistryParams, StdError>(params)
            })?;
        },
        SudoMsg::UpdateManagers { managers } => {
            REGISTRY_PARAMS.update(deps.storage, |mut params| {
                params.managers = managers;
                Ok::<RegistryParams, StdError>(params)
            })?;
        },
        SudoMsg::UpdateParams(params) => {
            REGISTRY_PARAMS.save(deps.storage, &params)?;
        },
    }
    Ok(Response::new().add_attributes(vec![("action", "admin_update")]))
}
