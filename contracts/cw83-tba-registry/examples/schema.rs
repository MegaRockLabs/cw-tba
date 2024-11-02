use cosmwasm_schema::write_api;
use cosmwasm_std::Binary;
use cw83_tba_registry::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

fn main() {
    write_api! {
        name: "cw83_tba_registry",
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg<Binary>,
        migrate: MigrateMsg,
    }
}
