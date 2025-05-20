use cosm_orc::orchestrator::cosm_orc::tokio_block;
use cosm_tome::chain::coin::Coin;
use cosm_tome::{chain::request::TxOptions, modules::bank::model::SendRequest};
use cosmwasm_std::{from_json, to_json_binary, BankMsg, Binary, CosmosMsg, Empty, WasmMsg};
use cw_ownable::Ownership;
use cw_tba::{ExecuteMsg, TokenInfo};
use test_context::test_context;

use cw82_tba_base::msg::{
    AssetsResponse, KnownTokensResponse, QueryMsg, Status,
};

use crate::helpers::helper::{
    create_simple_token_account, get_init_address, instantiate_collection, mint_token, send_token,
    SIMPLE_ACCOUNT_NAME,
};
use crate::helpers::{
    chain::Chain,
    helper::{can_execute, full_setup, wasm_query, wasm_query_typed},
};


#[test_context(Chain)]
#[test]
#[ignore]
fn can_execute_test(chain: &mut Chain) {
    let data = full_setup(chain).unwrap();

    assert!(
        can_execute(
            chain,
            &data.token_account,
            data.signer_address.clone(),
            BankMsg::Send {
                to_address: data.signer_address.clone(),
                amount: vec![],
            }
            .into()
        )
        .can_execute
    );

    assert!(
        !can_execute(
            chain,
            &data.token_account,
            String::from("not owner"),
            BankMsg::Send {
                to_address: data.signer_address.clone(),
                amount: vec![],
            }
            .into()
        )
        .can_execute
    );

    // general wasm
    assert!(
        !can_execute(
            chain,
            &data.token_account,
            data.signer_address.clone(),
            WasmMsg::Execute {
                contract_addr: String::from("any"),
                msg: Binary::from(b"any"),
                funds: vec![]
            }
            .into()
        )
        .can_execute
    );

    // stargate
    assert!(
        !can_execute(
            chain,
            &data.token_account,
            data.signer_address.clone(),
            CosmosMsg::Stargate { 
                type_url: String::default(),
                value: Binary::default()
            }
        )
        .can_execute
    );

    // authz
    assert!(
        !can_execute(
            chain,
            &data.token_account,
            data.signer_address.clone(),
            CosmosMsg::Stargate {
                type_url: String::from("/cosmos/authz/v1beta1/..."),
                value: Binary::default()
            }
        )
        .can_execute
    );
}
#[test_context(Chain)]
#[test]
#[ignore]
fn general_queries(chain: &mut Chain) {
    let data = full_setup(chain).unwrap();

    let res = wasm_query(chain, &data.token_account, &QueryMsg::Pubkey {});
    let pubkey = from_json::<Binary>(&res.unwrap().res.data.unwrap()).unwrap();

    assert_eq!(pubkey, data.public_key);

    let res = wasm_query(chain, &data.token_account, &QueryMsg::Status {});
    let status = from_json::<Status>(&res.unwrap().res.data.unwrap()).unwrap();

    assert_eq!(status, Status { frozen: false });

    let res = wasm_query(chain, &data.token_account, &QueryMsg::Ownership {});
    let ownership = from_json::<Ownership<String>>(&res.unwrap().res.data.unwrap()).unwrap();

    assert_eq!(ownership.owner, Some(data.signer_address));

    let res = wasm_query(chain, &data.token_account, &QueryMsg::Token {});
    let info = from_json::<TokenInfo>(&res.unwrap().res.data.unwrap()).unwrap();

    assert_eq!(
        info,
        TokenInfo {
            collection: data.collection,
            id: data.token_id
        }
    );

    let res = wasm_query(
        chain,
        &data.token_account,
        &QueryMsg::Assets {
            skip: None,
            limit: None,
        },
    );
    let assets = from_json::<AssetsResponse>(&res.unwrap().res.data.unwrap()).unwrap();

    assert_eq!(assets.balances, vec![]);
}

#[test_context(Chain)]
#[test]
#[ignore]
fn known_assets(chain: &mut Chain) {
    let data = full_setup(chain).unwrap();
    let user = chain.cfg.users[0].clone();

    let assets: AssetsResponse = wasm_query_typed(
        chain,
        &data.token_account,
        &QueryMsg::Assets {
            skip: None,
            limit: None,
        },
    )
    .unwrap();

    let tokens: KnownTokensResponse = wasm_query_typed(
        chain,
        &data.token_account,
        &QueryMsg::KnownTokens {
            skip: None,
            limit: None,
        },
    )
    .unwrap();

    assert_eq!(assets.balances, vec![]);
    assert_eq!(assets.tokens, vec![]);
    assert_eq!(tokens.len(), 0);

    let denom = chain.cfg.orc_cfg.chain_cfg.denom.clone();

    tokio_block(async {
        chain
            .orc
            .client
            .bank_send(
                SendRequest {
                    amounts: vec![Coin {
                        denom: denom.clone().parse().unwrap(),
                        amount: 1000u128,
                    }],
                    from: user.account.address.parse().unwrap(),
                    to: data.token_account.parse().unwrap(),
                },
                &user.key,
                &TxOptions {
                    timeout_height: None,
                    fee: None,
                    memo: String::default(),
                    sequence: None,
                },
            )
            .await
    })
    .unwrap();

    let assets: AssetsResponse = wasm_query_typed(
        chain,
        &data.token_account,
        &QueryMsg::Assets {
            skip: None,
            limit: None,
        },
    )
    .unwrap();

    assert_eq!(
        assets.balances,
        vec![cosmwasm_std::Coin {
            denom: denom.clone(),
            amount: 1000u128.into()
        }]
    );
}

#[test_context(Chain)]
#[test]
#[ignore]
fn know_tokens_on_recieve(chain: &mut Chain) {
    let data = full_setup(chain).unwrap();
    let user = chain.cfg.users[0].clone();

    let tokens: KnownTokensResponse = wasm_query_typed(
        chain,
        &data.token_account,
        &QueryMsg::KnownTokens {
            skip: None,
            limit: None,
        },
    )
    .unwrap();
    assert_eq!(tokens.len(), 0);

    let mint_res = mint_token(chain, "3".into(), user.account.address.clone(), &user.key).unwrap();

    let token_id = mint_res
        .res
        .find_event_tags("wasm".to_string(), "token_id".to_string())[0]
        .value
        .clone();

    let tokens: KnownTokensResponse = wasm_query_typed(
        chain,
        &data.token_account,
        &QueryMsg::KnownTokens {
            skip: None,
            limit: None,
        },
    )
    .unwrap();

    assert_eq!(tokens.len(), 0);

    send_token(
        chain,
        token_id.clone(),
        data.token_account.clone(),
        Binary::default(),
        &user.key,
    )
    .unwrap();

    let tokens: KnownTokensResponse = wasm_query_typed(
        chain,
        &data.token_account.clone(),
        &QueryMsg::KnownTokens {
            skip: None,
            limit: None,
        },
    )
    .unwrap();

    assert_eq!(tokens.len(), 1);

    let first = tokens.first().unwrap().clone();

    assert_eq!(
        first,
        TokenInfo {
            collection: data.collection,
            id: token_id
        }
    )
}

#[test_context(Chain)]
#[test]
#[ignore]
fn tokens_receving(chain: &mut Chain) {
    let data: crate::helpers::helper::FullSetupData = full_setup(chain).unwrap();
    let user = chain.cfg.users[0].clone();

    let tokens: KnownTokensResponse = wasm_query_typed(
        chain,
        &data.token_account,
        &QueryMsg::KnownTokens {
            skip: None,
            limit: None,
        },
    )
    .unwrap();
    assert_eq!(tokens.len(), 0);

    let token_id = "4".to_string();

    // mint direclty to the token account
    mint_token(
        chain,
        token_id.clone(),
        data.token_account.clone(),
        &user.key,
    )
    .unwrap();

    let tokens: KnownTokensResponse = wasm_query_typed(
        chain,
        &data.token_account,
        &QueryMsg::KnownTokens {
            skip: None,
            limit: None,
        },
    )
    .unwrap();

    // account does not know about the token
    assert_eq!(tokens.len(), 0);

    chain
        .orc
        .execute::<&str, ExecuteMsg>(
            SIMPLE_ACCOUNT_NAME,
            "acc_tokens_ack",
            &ExecuteMsg::UpdateKnownTokens {
                collection: data.collection.clone(),
                start_after: None,
                limit: None,
            },
            &user.key,
            vec![],
        )
        .unwrap();

    let tokens: KnownTokensResponse = wasm_query_typed(
        chain,
        &data.token_account.clone(),
        &QueryMsg::KnownTokens {
            skip: None,
            limit: None,
        },
    )
    .unwrap();

    assert_eq!(tokens.len(), 1);
} 


#[test_context(Chain)]
#[test]
#[ignore]
fn tokens_acknowlegement(chain: &mut Chain) {
    let data = full_setup(chain).unwrap();
    let user = chain.cfg.users[0].clone();

    // minting 3 token for token account
    for id in ["12", "13", "14", "15", "16"].into_iter() {
        mint_token(chain, id.to_string(), data.token_account.clone(), &user.key).unwrap();
    }

    // making the account aware of the tokens it owns
    chain
        .orc
        .execute::<&str, ExecuteMsg>(
            SIMPLE_ACCOUNT_NAME,
            "acc_tokens_ack",
            &ExecuteMsg::UpdateKnownTokens {
                collection: data.collection.clone(),
                start_after: None,
                limit: None,
            },
            &user.key,
            vec![],
        )
        .unwrap();

    // ----------------------------------------------------

    // Transfering to EOA after which there are only 2 left
    chain
        .orc
        .execute::<&str, ExecuteMsg>(
            SIMPLE_ACCOUNT_NAME,
            "acc_token_acc",
            &ExecuteMsg::TransferToken {
                collection: data.collection.clone(),
                token_id: "12".into(),
                recipient: user.account.address.clone(),
            },
            &user.key,
            vec![],
        )
        .unwrap();

    let tokens: KnownTokensResponse = wasm_query_typed(
        chain,
        &data.token_account.clone(),
        &QueryMsg::KnownTokens {
            skip: None,
            limit: None,
        },
    )
    .unwrap();
    assert_eq!(tokens.len(), 4);

    // ----------------------------------------------------

    // sending to itself should be fine but not change anything
    chain
        .orc
        .execute::<&str, ExecuteMsg>(
            SIMPLE_ACCOUNT_NAME,
            "acc_token_acc",
            &ExecuteMsg::SendToken {
                collection: data.collection.clone(),
                token_id: "13".into(),
                contract: data.token_account.clone(),
                msg: Binary::default(),
            },
            &user.key,
            vec![],
        )
        .unwrap();

    let tokens: KnownTokensResponse = wasm_query_typed(
        chain,
        &data.token_account.clone(),
        &QueryMsg::KnownTokens {
            skip: None,
            limit: None,
        },
    )
    .unwrap();
    assert_eq!(tokens.len(), 4);

    // ----------------------------------------------------

    // create account with newly received token 2
    let create_res = create_simple_token_account(
        chain,
        data.collection.clone(),
        "12".to_string(),
        data.public_key,
        &user.key,
    )
    .unwrap();

    let second_ta = get_init_address(create_res.res);

    // sending to an nft to second token account
    chain
        .orc
        .execute::<&str, ExecuteMsg>(
            SIMPLE_ACCOUNT_NAME,
            "acc_token_acc",
            &ExecuteMsg::SendToken {
                collection: data.collection.clone(),
                token_id: "13".into(),
                contract: second_ta.clone(),
                msg: Binary::default(),
            },
            &user.key,
            vec![],
        )
        .unwrap();

    // first token account now only knows about 3 tokens
    let tokens: KnownTokensResponse = wasm_query_typed(
        chain,
        &data.token_account.clone(),
        &QueryMsg::KnownTokens {
            skip: None,
            limit: None,
        },
    )
    .unwrap();
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens.first().unwrap().id, "14".to_string());

    // second token account only knows about 1 token "3"
    let tokens: KnownTokensResponse = wasm_query_typed(
        chain,
        &second_ta.clone(),
        &QueryMsg::KnownTokens {
            skip: None,
            limit: None,
        },
    )
    .unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens.first().unwrap().id, "13".to_string());

    // ----------------------------------------------------

    // making the token account forget about the tokens it owns
    chain
        .orc
        .execute::<&str, ExecuteMsg>(
            SIMPLE_ACCOUNT_NAME,
            "acc_token_acc",
            &ExecuteMsg::ForgetTokens {
                collection: data.collection.clone(),
                token_ids: vec!["14".to_string()],
            },
            &user.key,
            vec![],
        )
        .unwrap();

    let tokens: KnownTokensResponse = wasm_query_typed(
        chain,
        &data.token_account.clone(),
        &QueryMsg::KnownTokens {
            skip: None,
            limit: None,
        },
    )
    .unwrap();
    assert_eq!(tokens.len(), 2);

    // forget about the whole collection
    chain
        .orc
        .execute::<&str, ExecuteMsg>(
            SIMPLE_ACCOUNT_NAME,
            "acc_token_acc",
            &ExecuteMsg::ForgetTokens {
                collection: data.collection.clone(),
                token_ids: vec![],
            },
            &user.key,
            vec![],
        )
        .unwrap();

    let tokens: KnownTokensResponse = wasm_query_typed(
        chain,
        &data.token_account.clone(),
        &QueryMsg::KnownTokens {
            skip: None,
            limit: None,
        },
    )
    .unwrap();
    assert_eq!(tokens.len(), 0);

    // ----------------------------------------------------

    // Token Account balance is still ok (just unaware of it)
    let res: cw721::TokensResponse = wasm_query_typed(
        chain,
        &data.collection,
        &cw721_base::QueryMsg::<Empty>::Tokens {
            owner: data.token_account.clone(),
            start_after: None,
            limit: None,
        },
    )
    .unwrap();
    assert_eq!(res.tokens.len(), 3);
    assert_eq!(
        res.tokens,
        vec![String::from("14"), String::from("15"), String::from("16"),]
    );

    // Other balances ok
    let res: cw721::TokensResponse = wasm_query_typed(
        chain,
        &data.collection,
        &cw721_base::QueryMsg::<Empty>::Tokens {
            owner: second_ta.clone(),
            start_after: None,
            limit: None,
        },
    )
    .unwrap();
    assert_eq!(res.tokens.len(), 1);
    assert_eq!(res.tokens.first().unwrap(), &String::from("13"));

    let res: cw721::TokensResponse = wasm_query_typed(
        chain,
        &data.collection,
        &cw721_base::QueryMsg::<Empty>::Tokens {
            owner: user.account.address.clone(),
            start_after: None,
            limit: None,
        },
    )
    .unwrap();
    assert_eq!(res.tokens.len(), 3);
}


#[test_context(Chain)]
#[test]
#[ignore]
fn direct_mint(chain: &mut Chain) {
    let data = full_setup(chain).unwrap();
    let user = chain.cfg.users[0].clone();

    let init_res =
        instantiate_collection(chain, data.token_account.clone(), None, &user.key).unwrap();

    let collection = get_init_address(init_res.res);
    let mint_msg: cw721_base::ExecuteMsg<Option<Empty>, Empty> = cw721_base::ExecuteMsg::Mint {
        token_id: "1".to_string(),
        owner: data.token_account.clone(),
        token_uri: None,
        extension: None,
    };

    let account_msg = ExecuteMsg::MintToken {
        minter: collection.clone(),
        msg: to_json_binary(&mint_msg).unwrap(),
    };

    chain
        .orc
        .execute::<&str, ExecuteMsg>(
            SIMPLE_ACCOUNT_NAME,
            "acc_tokens_mint",
            &account_msg,
            &user.key,
            vec![],
        ).unwrap();
    

    let tokens : KnownTokensResponse = wasm_query_typed(
        chain,
        &data.token_account,
        &QueryMsg::KnownTokens { skip: None, limit: None }
    ).unwrap();

    assert_eq!(tokens.len(), 1);


    let res : cw721::TokensResponse = wasm_query_typed(
        chain,
        &collection,
        &cw721_base::QueryMsg::<Empty>::Tokens {
            owner: data.token_account.clone(),
            start_after: None,
            limit: None
        }
    ).unwrap();
    assert_eq!(res.tokens.len(), 1);
}
