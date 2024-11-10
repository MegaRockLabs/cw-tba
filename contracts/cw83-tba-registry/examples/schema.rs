use cosmwasm_schema::write_api;
use cw83_tba_registry::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use saa::CredentialData;

fn main() {
    write_api! {
        name: "cw83_tba_registry",
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg<CredentialData>,
        migrate: MigrateMsg,
    }
}
