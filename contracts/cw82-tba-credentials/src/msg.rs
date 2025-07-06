pub type ContractResult = Result<cosmwasm_std::Response, crate::error::ContractError>;
pub use cw_tba::{InstantiateAccountMsg as InstantiateMsg, QueryMsg};
pub use cosmwasm_std::Binary as MigrateMsg;

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub enum SudoMsg {
    #[cfg(feature = "archway")]
    CwGrant(crate::grants::CwGrant),
}
