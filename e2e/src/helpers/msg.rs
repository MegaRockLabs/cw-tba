use cosmrs::tx::AccountNumber;
use cosmwasm_schema::cw_serde;


#[cw_serde]
pub struct ProxyInstantiateMsg {
    pub admins: Vec<String>,
    pub mutable: bool,
}


/* 
  readonly chain_id: string;
  readonly account_number: string;
  readonly sequence: string;
  readonly fee: StdFee;
  readonly msgs: readonly AminoMsg[];
  readonly memo: string;
 */

 
#[cw_serde]
pub struct SignDoc {
    /// `body_bytes` is protobuf serialization of a transaction [`Body`] that matches the
    /// representation in a [`Raw`] transaction.
    pub body_bytes: Vec<u8>,

    /// `auth_info_bytes` is a protobuf serialization of an [`AuthInfo`] that matches the
    /// representation in a [`Raw`].
    pub auth_info_bytes: Vec<u8>,

    /// `chain_id` is the unique identifier of the chain this transaction targets.
    ///
    /// It prevents signed transactions from being used on another chain by an
    /// attacker.
    pub chain_id: String,

    /// `account_number` is the account number of the account in state
    pub account_number: AccountNumber,
}