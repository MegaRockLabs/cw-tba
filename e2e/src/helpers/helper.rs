use std::str::FromStr;
use super::chain::Chain;
use cosm_orc::orchestrator::cosm_orc::tokio_block;
use cosm_orc::orchestrator::error::{ProcessError, CosmwasmError};
use cosm_orc::orchestrator::{Coin as OrcCoin, ExecResponse, Address, ChainTxResponse, QueryResponse, Denom};
use cosm_orc::orchestrator::{InstantiateResponse, SigningKey};
use cosm_tome::chain::request::TxOptions;
use cosm_tome::clients::client::CosmosClient;
use cosm_tome::modules::bank::model::SendRequest;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Timestamp, Empty, CosmosMsg, Binary, from_json, Coin,};

use cw1::CanExecuteResponse;
use cw82_base::msg::QueryMsg;
use cw83_base::msg::InstantiateMsg;
use serde::Serialize;
use serde::de::DeserializeOwned;
use cw_tba::{TokenInfo, MigrateAccountMsg, CreateAccountMsg};

// contract names used by cosm-orc to register stored code ids / instantiated addresses:
pub const COLLECTION_NAME        : &str = "cw721_base";
pub const BASE_REGISTRY_NAME     : &str = "cw83_base";
pub const BASE_ACOUNT_NAME       : &str = "cw82_base";


pub const MAX_TOKENS: u32 = 10_000;
pub const CREATION_FB_FEE: u128 = 100_000_000;
pub const MINT_PRICE: u128 = 100_000_000;


pub fn creation_fees_wasm<C: CosmosClient> (chain: &Chain<C>) -> Vec<Coin> {
    vec![Coin {
        denom: chain.cfg.orc_cfg.chain_cfg.denom.clone(),
        amount: CREATION_FB_FEE.into(),
    }]
}

pub fn creation_fee<C: CosmosClient>(chain: &Chain<C>) -> Vec<OrcCoin> {
    vec![OrcCoin {
        denom: Denom::from_str(&chain.cfg.orc_cfg.chain_cfg.denom).unwrap(),
        amount: CREATION_FB_FEE.into(),
    }]
}

pub fn instantiate_registry<C: CosmosClient>(
    chain: &mut Chain<C>,
    creator_addr: String,
    key: &SigningKey
) -> Result<InstantiateResponse, ProcessError> {
    
    let account_id = chain.orc.contract_map.code_id(BASE_ACOUNT_NAME)?;

    chain.orc.instantiate(
        BASE_REGISTRY_NAME,
        "registry_instantiate",
        &InstantiateMsg {
            params: cw_tba::RegistryParams {
                allowed_code_ids: vec![account_id],
                creation_fees: creation_fees_wasm(&chain),
                managers: vec![],
                extension: Empty {}
            },
        },
        key,
        Some(creator_addr.parse().unwrap()),
        vec![],
    )
}


pub fn instantiate_collection<C: CosmosClient>(
    chain: &mut Chain<C>,
    minter: String,
    nonce: Option<&str>,
    key: &SigningKey,
) -> Result<InstantiateResponse, ProcessError> {
    // let infos: Vec<(String, DeployInfo)> =  chain.cfg.orc_cfg.contract_deploy_info.clone().into_iter().collect();

    chain.orc.instantiate(
        COLLECTION_NAME,
        &"collection_instantiate",
        &cw721_base::InstantiateMsg {
            name: "test".to_string() + nonce.unwrap_or_default(),
            symbol: "test".to_string() + nonce.unwrap_or_default(),
            minter: minter.clone(),
        },
        key,
        Some(Address::from_str(&minter).unwrap()),
        vec![],
    )
}



pub fn mint_token<C: CosmosClient>(
    chain: &mut Chain<C>,
    token_id: String,
    owner: String,
    key: &SigningKey,
) -> Result<ExecResponse, ProcessError> {

    let mint_msg = cw721_base::ExecuteMsg::<Option<Empty>, Empty>::Mint { 
        token_id, 
        owner, 
        token_uri: None, 
        extension: None
    };

    chain.orc.execute(
        COLLECTION_NAME,
        "token_mint",
        &mint_msg,
        key,
        vec![],
    )
}


pub fn send_token<C: CosmosClient>(
    chain: &mut Chain<C>,
    token_id: String,
    recipient: String,
    msg: Binary,
    key: &SigningKey,
) -> Result<ExecResponse, ProcessError> {

    let send_msg = cw721_base::ExecuteMsg::<Option<Empty>, Empty>::SendNft { 
        contract: recipient, 
        token_id, 
        msg
    };

    chain.orc.execute(
        COLLECTION_NAME,
        "send_nft_acknowledge",
        &send_msg,
        key,
        vec![],
    )
}




pub fn create_token_account<C: CosmosClient>(
    chain: &mut Chain<C>,
    token_contract: String,
    token_id: String,
    pubkey: Binary,
    key: &SigningKey,
) -> Result<ExecResponse, ProcessError> {

    let chain_id = chain.cfg.orc_cfg.chain_cfg.chain_id.clone();

    let init_msg = cw_tba::TokenAccount { 
        token_info: TokenInfo {
            collection: token_contract,
            id: token_id,
        }, 
        account_data: pubkey,
        create_for: None, 
    };


    let code_id = chain.orc.contract_map.code_id(BASE_ACOUNT_NAME)?;

    chain.orc.execute(
        BASE_REGISTRY_NAME, 
        "registry_create_account", 
        &cw83_base::msg::ExecuteMsg::CreateAccount(
            CreateAccountMsg {
                code_id,
                chain_id,
                msg: init_msg,
            }
        ), 
        key, 
        creation_fee(&chain)
    )
}


pub fn reset_token_account<C: CosmosClient>(
    chain: &mut Chain<C>,
    token_contract: String,
    token_id: String,
    pubkey: Binary,
    key: &SigningKey,
) -> Result<ExecResponse, ProcessError> {

    let chain_id = chain.cfg.orc_cfg.chain_cfg.chain_id.clone();

    let init_msg = cw_tba::TokenAccount { 
        token_info: TokenInfo {
            collection: token_contract,
            id: token_id,
        }, 
        account_data: pubkey,
        create_for: None,
    };

    let code_id = chain.orc.contract_map.code_id(BASE_ACOUNT_NAME)?;

    chain.orc.execute(
        BASE_REGISTRY_NAME, 
        "registry_reset_account", 
        &cw83_base::msg::ExecuteMsg::ResetAccount(
            CreateAccountMsg {
                code_id,
                chain_id,
                msg: init_msg,
            }
        ), 
        key, 
        creation_fee(&chain)
    )
}



pub fn migrate_token_account<C: CosmosClient>(
    chain: &mut Chain<C>,
    token_contract: String,
    token_id: String,
    key: &SigningKey,
) -> Result<ExecResponse, ProcessError> {

    let code_id = chain.orc.contract_map.code_id(BASE_ACOUNT_NAME)?;

    let migrate_msg = cw83_base::msg::ExecuteMsg::MigrateAccount { 
        token_info: TokenInfo {
            collection: token_contract,
            id: token_id,
        }, 
        new_code_id: code_id, 
        msg: MigrateAccountMsg { params: Box::new(None) }
    };


    chain.orc.execute(
        BASE_REGISTRY_NAME, 
        "registry_reset_account", 
        &migrate_msg, 
        key, 
        vec![]
    )
}


#[cw_serde]
pub struct FullSetupData {
    pub registry: String,
    pub collection: String,
    pub token_id: String,
    pub token_account: String,

    pub signer_mnemonic: String,
    pub public_key: Binary,

    pub user_address: String,
}


pub fn get_init_address(
    res: ChainTxResponse
) -> String {
    res
        .find_event_tags(
            "instantiate".to_string(), 
            "_contract_address".to_string()
        )[0].value.clone()
}


pub fn full_setup<C: CosmosClient>(
    chain: &mut Chain<C>,
) -> Result<FullSetupData, ProcessError> {

    let _start_time = latest_block_time(chain).plus_seconds(60);


    let user: super::chain::SigningAccount = chain.cfg.users[0].clone();
    let user_address = user.account.address.clone();

    let reg_init = instantiate_registry(chain, user_address.clone(), &user.key).unwrap();
    
    let registry = get_init_address(reg_init.res);

    let init_res = instantiate_collection(
        chain, 
        user.account.address.clone(), 
        None,
        &user.key
    ).unwrap();

    let collection  = get_init_address(init_res.res);
    chain.orc.contract_map.add_address(COLLECTION_NAME, collection.clone()).unwrap();
    
    let mint_res = mint_token(
        chain, 
        "1".to_string(),
        user.account.address.clone(), 
        &user.key
    ).unwrap();


    let token_id = mint_res
                .res
                .find_event_tags(
                    "wasm".to_string(), 
                    "token_id".to_string()
                )[0].value.clone();
            

    let pubkey : Binary = user.key.public_key().unwrap().to_bytes().into();

    let create_res = create_token_account(
        chain, 
        collection.clone(),
        token_id.clone(),
        pubkey.clone(),
        &user.key
    ).unwrap();


    let token_account = get_init_address(create_res.res);
    chain.orc.contract_map.add_address(BASE_ACOUNT_NAME, token_account.clone()).unwrap();

    Ok(FullSetupData {
        registry,
        collection,
        token_id,
        token_account,

        signer_mnemonic: user.account.mnemonic,
        public_key: pubkey,

        user_address
    })

}
 



pub fn wasm_query<C : CosmosClient, S: Serialize>(
    chain: &mut Chain<C>,
    address: &String,
    msg: &S
) -> Result<QueryResponse, CosmwasmError> {

    let res = tokio_block(async { 
        chain.orc.client.wasm_query(
            Address::from_str(&address)?,
            msg
        )
        .await }
    );

    res
}

pub fn wasm_query_typed<C, R, S> (
    chain: &mut Chain<C>,
    address: &String,
    msg: &S
) -> Result<R, CosmwasmError> 
where 
      C: CosmosClient,
      S: Serialize,
      R: DeserializeOwned
{
    let res = tokio_block(async { 
        chain.orc.client.wasm_query(
            Address::from_str(&address)?,
            msg
        )
        .await }
    )?;


    let res : R = from_json(
        &res.res.data.unwrap()
    ).unwrap();

    Ok(res)
}



pub fn query_token_owner<C: CosmosClient>(
    chain: &mut Chain<C>,
    collection: String,
    token_id: String,
) -> Result<cw721::OwnerOfResponse, CosmwasmError> {

    let res = wasm_query(
        chain,
        &collection,
        &cw721::Cw721QueryMsg::OwnerOf {
            token_id, include_expired: None
        }
    ).unwrap();

    let owner_res : cw721::OwnerOfResponse = from_json(
        &res.res.data.unwrap()
    ).unwrap();

    Ok(owner_res)
}
 


// gen_users will create `num_users` random SigningKeys
// and then transfer `init_balance` of funds to each of them.
pub fn gen_users<C: CosmosClient>(
    chain: &mut Chain<C>,
    num_users: u32,
    init_balance: u128,
    denom: Option<&String>,
) -> Vec<SigningKey> {
    let cfg = &chain.cfg.orc_cfg.chain_cfg;
    let from_user = &chain.cfg.users[1];

    let mut users = vec![];
    for n in 0..num_users {
        users.push(SigningKey::random_mnemonic(n.to_string(), cfg.derivation_path.clone()));
    }

    let mut reqs = vec![];
    for user in &users {
        let mut amounts = vec![OrcCoin {
            amount: init_balance,
            denom: cfg.denom.parse().unwrap(),
        }];
        // add extra denom if specified
        if let Some(denom) = denom {
            amounts.push(OrcCoin {
                amount: init_balance,
                denom: denom.parse().unwrap(),
            });
        }
        reqs.push(SendRequest {
            from: from_user.account.address.parse().unwrap(),
            to: user.to_addr(&cfg.prefix).unwrap(),
            amounts,
        });
    }

    tokio_block(
        chain
            .orc
            .client
            .bank_send_batch(reqs, &from_user.key, &TxOptions::default()),
    )
    .unwrap();

    users
}

pub fn latest_block_time<C: CosmosClient>(chain: &Chain<C>) -> Timestamp {
    let now = tokio_block(chain.orc.client.tendermint_query_latest_block())
        .unwrap()
        .block
        .header
        .unwrap()
        .time
        .unwrap();

    Timestamp::from_seconds(now.seconds.try_into().unwrap())
}



pub fn can_execute<C: CosmosClient>(
    chain: &mut Chain<C>, 
    token_account: &String, 
    sender: String, 
    msg: CosmosMsg
) -> CanExecuteResponse {
    let res = wasm_query(
        chain, 
        token_account, 
        &QueryMsg::CanExecute { 
            sender: sender, 
            msg: msg.into(), 
        }
    ).unwrap();
    
    from_json(
        &res.res.data.unwrap()
    ).unwrap()
}