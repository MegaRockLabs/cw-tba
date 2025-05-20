use cosmwasm_schema::cw_serde;
use cosmwasm_std::Response;
use cw_tba::{InstantiateAccountMsg, MigrateAccountMsg};
use crate::error::ContractError;
pub use cw_tba::QueryMsg;



#[cw_serde]
pub enum SudoMsg {
    #[cfg(feature = "archway")]
    CwGrant(crate::grants::CwGrant)
}


pub type InstantiateMsg = InstantiateAccountMsg;
pub type MigrateMsg = MigrateAccountMsg;
pub type ContractResult = Result<Response, ContractError>;


