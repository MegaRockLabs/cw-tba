use test_context::test_context;

use crate::helpers::{
    chain::Chain,
    helper::{
        full_setup, instantiate_collection, mint_token, query_token_owner,
    },
};


#[test_context(Chain)]
#[test]
#[ignore]
fn test_mint_token(chain: &mut Chain) {
    let user = chain.cfg.users[0].clone();

    let init_res =
        instantiate_collection(chain, user.account.address.clone(), None, &user.key).unwrap();

    let tags = init_res
        .res
        .find_event_tags("instantiate".to_string(), "_contract_address".to_string());

    let col_address = tags[0].value.clone();

    let mint_res = mint_token(
        chain,
        "1".to_string(),
        user.account.address.clone(),
        &user.key,
    )
    .unwrap();

    let token_id = mint_res
        .res
        .find_event_tags("wasm".to_string(), "token_id".to_string())[0]
        .value
        .clone();

    let owner_res = query_token_owner(chain, col_address, token_id).unwrap();

    assert_eq!(user.account.address, owner_res.owner)
}



#[test_context(Chain)]
#[test]
#[ignore]
fn test_create_token_accounts(chain: &mut Chain) {
    let data = full_setup(chain).unwrap();
    assert!(data.token_account.len() > 0);
}
