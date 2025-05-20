use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Coin, CosmosMsg, DepsMut, Env, Response};
use cw_auths::has_natives;

use crate::{error::ContractError};


#[cw_serde]
pub struct CwGrant {
    pub fee_requested: Vec<Coin>,
    pub msgs: Vec<CwGrantMessage>,
}


#[cw_serde]
pub struct CwGrantMessage {
    pub sender: String,
    pub type_url: String,
    pub msg: Binary,
}



pub fn cwfee_grant(deps: DepsMut, env: Env, msg: CwGrant) -> Result<Response, ContractError> {
    let with_caller = has_natives(deps.storage);
    
    let owner = cw_ownable::get_ownership(deps.storage)?
        .owner
        .ok_or(ContractError::NoOwnerCred {})?;

    let owner = owner.as_str();

    for m in &msg.msgs {
        if with_caller && owner == m.sender.as_str() {
            continue;
        }
        if m.type_url == "cosmwasm.wasm.v1.MsgExecuteContract" {
            let contract = anybuf::Bufany::deserialize(&m.msg)
                .map_err(|e| ContractError::Generic(e.to_string()))?
                .string(2).unwrap_or_default();
            if env.contract.address.to_string() == contract {
                continue;
            }
        }
        return Err(ContractError::Unauthorized("Not ellible for a cwfee grant".to_string()));
    }

    Ok(Response::default())
}



pub fn register_granter_msg(env: &Env) -> Result<CosmosMsg, ContractError> {

    let register_stargate_msg = CosmosMsg::Stargate {
        type_url: "/archway.cwfees.v1.MsgRegisterAsGranter".to_string(),
        value: Binary(
            anybuf::Anybuf::new().
            append_string(1, env.contract.address.to_string())
            .into_vec(),
        ),
    };

    Ok(register_stargate_msg)
}