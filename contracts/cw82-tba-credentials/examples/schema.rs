use cosmwasm_schema::write_api;
use cw82_tba_credentials::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

fn main() {
    write_api! {
        name: "cw82_tba_credentials",
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg,
        query: QueryMsg,
    }
}
