use cosmwasm_schema::write_api;
use cw82_tba_base::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

fn main() {
    write_api! {
        name: "cw82_tba_base",
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg,
        query: QueryMsg,
    }
}
