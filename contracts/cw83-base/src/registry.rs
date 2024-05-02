use cosmwasm_schema::{cw_serde, serde::Serialize};
use cosmwasm_std::{Deps, to_json_binary, Binary, Addr, Coin, StdResult, CosmosMsg, SubMsg, ReplyOn};
use cw83::{Cw83RegistryBase, CREATE_ACCOUNT_REPLY_ID};
use cw_tba::{TokenInfo, InstantiateAccountMsg};


pub fn construct_label(
    info: &TokenInfo,
    serial: Option<u64>
) -> String {
    let base =  format!("{}-{}-account", info.collection, info.id);
    match serial {
        Some(s) => format!("{}-{}", base, s),
        None => base
    }
}


#[cw_serde]
pub struct Cw83TokenRegistryContract(pub Addr);

impl Cw83TokenRegistryContract {
    
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    fn cw83_wrap(&self) -> Cw83RegistryBase {
        Cw83RegistryBase(self.addr())
    }

    fn init_binary<T: Serialize>(
        &self,
        owner: String,
        token_info: TokenInfo,
        account_data: T,
    ) -> StdResult<Binary> {

        let msg = InstantiateAccountMsg {
            owner,
            token_info,
            account_data,
        };

        to_json_binary(&msg)
    }

    pub fn create_account_init_msg<T: Serialize>(
        &self, 
        code_id: u64, 
        owner: String,
        token_info: TokenInfo,
        account_data: T,
        funds: Vec<Coin>,
        serial: Option<u64>
    ) -> StdResult<CosmosMsg> {

        self.cw83_wrap().create_account_init_msg(
            code_id,
            self.init_binary(
                owner,
                token_info.clone(),
                account_data
            )?,
            funds,
            construct_label(&token_info, serial)
        )
    }

    pub fn create_account_sub_msg<T: Serialize>(
        &self, 
        code_id: u64, 
        owner: String,
        token_info: TokenInfo,
        account_data: T,
        funds: Vec<Coin>,
        serial: Option<u64>
    ) -> StdResult<SubMsg> {

        Ok(SubMsg {
            id: CREATE_ACCOUNT_REPLY_ID,
            msg: self.create_account_init_msg(
                code_id,
                owner,
                token_info,
                account_data,
                funds,
                serial
            )?,
            reply_on: ReplyOn::Success,
            gas_limit: None
        })
    }
    
    pub fn supports_interface(
        &self,
        deps: Deps,
    ) -> StdResult<bool> {
        self.cw83_wrap().supports_interface(&deps.querier)
    }

}