use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Coin, CosmosMsg, DepsMut, Env, Response};
use cw_storage_plus::Item;

use crate::{error::ContractError, state::{OWNER, WITH_CALLER}};

pub static GRANT_TEST: Item<CwGrantMessage> = Item::new("g");



#[cw_serde]
pub enum SudoMsg {
    CwGrant(CwGrant)
}


#[cw_serde]
pub enum GrantQuertMsg {
    Test {}
}

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






pub fn sudo_grant(deps: DepsMut, _env: Env, msg: CwGrant) -> Result<Response, ContractError> {
    let with_caller = WITH_CALLER.load(deps.storage)?;
    let owner = OWNER.load(deps.storage)?;


    for m in &msg.msgs {
        GRANT_TEST.save(deps.storage, &m)?;

        if with_caller && m.sender == owner {
            continue;
        }

    }

    Ok(Response::default())
}


#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRegisterAsGranter {
    #[prost(string, tag = "1")]
    pub granting_contract: ::prost::alloc::string::String,
}


pub fn register_granter_msg(env: &Env) -> Result<CosmosMsg, ContractError> {
    let regsiter_msg = MsgRegisterAsGranter {
        granting_contract: env.contract.address.to_string(),
    };

    let register_stargate_msg = CosmosMsg::Stargate {
        type_url: "/archway.cwfees.v1.MsgRegisterAsGranter".to_string(),
        value: Binary::from(prost::Message::encode_to_vec(&regsiter_msg)),
    };

    Ok(register_stargate_msg)
}