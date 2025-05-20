use cw82_tba_base::msg::{InstantiateMsg, MigrateMsg, QueryMsg};
use cw_tba::ExecuteMsg;

fn main() {
    cosmwasm_schema::write_api! {
        name: "cw82_tba_base",
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg,
        query: QueryMsg,
    }
}
