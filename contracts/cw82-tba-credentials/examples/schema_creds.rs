use cw82_tba_credentials::msg::{InstantiateMsg, MigrateMsg, QueryMsg};
use cw_tba::ExecuteMsg;

fn main() {
    cosmwasm_schema::write_api! {
        name: "cw82_tba_credentials",
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg,
        query: QueryMsg,
    }
}
