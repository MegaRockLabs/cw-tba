use super::chain::{Chain, SigningAccount};
use cosm_orc::orchestrator::cosm_orc::tokio_block;
use cosm_orc::orchestrator::error::{CosmwasmError, ProcessError};
use cosm_orc::orchestrator::{Address, ChainTxResponse, Denom, ExecResponse, QueryResponse, Coin as OrcCoin};
use cosm_orc::orchestrator::{InstantiateResponse, SigningKey};
use cosm_tome::chain::request::TxOptions;
use cosm_tome::clients::client::CosmosClient;
use cosm_tome::modules::bank::model::SendRequest;
use cosm_tome::signing_key::key::mnemonic_to_signing_key;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{from_json, to_json_binary, Binary, Coin, CosmosMsg, Empty, Timestamp};
use cw_auths::saa_types::utils::cosmos::preamble_msg_arb_036;
use cw_auths::saa_types::msgs::MsgDataToSign;
use std::str::FromStr;


use cw_tba::{CreateAccountMsg, MigrateAccountMsg, TokenInfo};
use cw_auths::saa_types::{CosmosArbitrary, Credential, CredentialData};
use serde::de::DeserializeOwned;
use serde::Serialize;


use cw1::CanExecuteResponse;
use cw82_tba_base::msg::QueryMsg;


// contract names used by cosm-orc to register stored code ids / instantiated addresses:
pub const BASE_REGISTRY_NAME: &str = "cw83_tba_registry";
pub const COLLECTION_NAME: &str = "cw721_base";

pub const SIMPLE_ACCOUNT_NAME: &str = "cw82_tba_base";
pub const CRED_ACOUNT_NAME: &str = "cw82_tba_credentials";

pub const MAX_TOKENS: u32 = 10_000;
pub const CREATION_FB_FEE: u128 = 100_000_000;
pub const MINT_PRICE: u128 = 100_000_000;


pub fn creation_fees_wasm<C: CosmosClient>(chain: &Chain<C>) -> Vec<Coin> {
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
    key: &SigningKey,
) -> Result<InstantiateResponse, ProcessError> {
    let map = &chain.orc.contract_map;

    chain.orc.instantiate(
        BASE_REGISTRY_NAME,
        "registry_instantiate",
        &cw83_tba_registry::msg::InstantiateMsg {
            params: cw_tba::RegistryParams {
                allowed_code_ids: vec![
                    map.code_id(SIMPLE_ACCOUNT_NAME)?,
                    map.code_id(CRED_ACOUNT_NAME)?,
                ],
                creation_fees: creation_fees_wasm(&chain),
                managers: vec![],
                extension: None,
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
        extension: None,
    };

    chain
        .orc
        .execute(COLLECTION_NAME, "token_mint", &mint_msg, key, vec![])
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
        msg: msg.to_vec().into(),
    };

    chain.orc.execute(
        COLLECTION_NAME,
        "send_nft_acknowledge",
        &send_msg,
        key,
        vec![],
    )
}


pub fn create_simple_token_account<C: CosmosClient>(
    chain: &mut Chain<C>,
    token_contract: String,
    token_id: String,
    pubkey: Binary,
    key: &SigningKey,
) -> Result<ExecResponse, ProcessError> {
    let chain_id = chain.cfg.orc_cfg.chain_cfg.chain_id.clone();

    let account_data = CredentialData {
        credentials: vec![CosmosArbitrary { 
            pubkey,
            signature: Binary::default(),
            message: Binary::default(),
            hrp: None
        }.into()],
        use_native: None,
        primary_index: None,
    };

    let init_msg = cw_tba::TokenAccount {
        token_info: TokenInfo {
            collection: token_contract,
            id: token_id,
        },
        actions: None,
        create_for: None,
        account_data,
    };

    let code_id = chain.orc.contract_map.code_id(SIMPLE_ACCOUNT_NAME)?;

    chain.orc.execute(
        BASE_REGISTRY_NAME,
        "registry_create_account",
        &cw83_tba_registry::msg::ExecuteMsg::CreateAccount(CreateAccountMsg {
            code_id,
            chain_id,
            msg: init_msg,
        }),
        key,
        creation_fee(&chain),
    )
}



pub fn get_cred_data<C: CosmosClient, M : Serialize>(
    chain: &mut Chain<C>,
    user: &SigningAccount,
    messages: Vec<M>,
) -> CredentialData {
    let chain_id = chain.cfg.orc_cfg.chain_cfg.chain_id.clone();
    let hrp = chain.cfg.orc_cfg.chain_cfg.prefix.clone();
    let sk = mnemonic_to_signing_key(&user.account.mnemonic, &user.key.derivation_path).unwrap();
    let registry = chain.orc.contract_map.address(BASE_REGISTRY_NAME).unwrap();
    
    let message = MsgDataToSign {
        chain_id: chain_id.clone(),
        contract_address: registry,
        messages,
        nonce: 0u64.into(),
    };

    let cred = Credential::CosmosArbitrary(CosmosArbitrary {
        pubkey: sk.public_key().to_bytes().into(),
        signature: sk.sign(
            &preamble_msg_arb_036(
                &user.account.address, 
                &to_json_binary(&message).unwrap().to_base64()
            ).as_bytes()
        ).unwrap().to_vec().into(),
        message: to_json_binary(&message).unwrap().into(),
        hrp: Some(hrp.into()),
    });

    CredentialData {
        credentials: vec![cred],
        use_native: None,
        primary_index: None,
    }
}


pub fn create_cred_token_account<C: CosmosClient>(
    chain: &mut Chain<C>,
    token_contract: String,
    token_id: String,
    user: &SigningAccount
) -> Result<ExecResponse, ProcessError> {
    let chain_id = chain.cfg.orc_cfg.chain_cfg.chain_id.clone();
    //let denom = chain.cfg.orc_cfg.chain_cfg.denom.clone();

    let account_data = get_cred_data(chain, user, Vec::<String>::new());
    
    let init_msg = cw_tba::TokenAccount {
        token_info: TokenInfo {
            collection: token_contract,
            id: token_id,
        },
        actions: None,
        create_for: None,
        account_data,
    };

    let code_id = chain.orc.contract_map.code_id(CRED_ACOUNT_NAME)?;

    chain.orc.execute(
        BASE_REGISTRY_NAME,
        "registry_create_cred_account",
        &cw83_tba_registry::msg::ExecuteMsg::CreateAccount(
            CreateAccountMsg {
                code_id,
                chain_id,
                msg: init_msg
            }),
        &user.key,
        creation_fee(&chain),
    )
}


pub fn reset_simple_token_account<C: CosmosClient>(
    chain: &mut Chain<C>,
    token_contract: String,
    token_id: String,
    pubkey: Binary,
    key: &SigningKey,
) -> Result<ExecResponse, ProcessError> {

    let chain_id = chain.cfg.orc_cfg.chain_cfg.chain_id.clone();

    let account_data = CredentialData {
        credentials: vec![CosmosArbitrary { 
            pubkey,
            signature: Binary::default(),
            message: Binary::default(),
            hrp: None
        }.into()],
        use_native: None,
        primary_index: None,
    };

    let init_msg = cw_tba::TokenAccount {
        token_info: TokenInfo {
            collection: token_contract,
            id: token_id,
        },
        actions: None,
        account_data,
        create_for: None,
    };

    let code_id = chain.orc.contract_map.code_id(SIMPLE_ACCOUNT_NAME)?;

    chain.orc.execute(
        BASE_REGISTRY_NAME,
        "registry_reset_account",
        &cw83_tba_registry::msg::ExecuteMsg::ResetAccount(CreateAccountMsg {
            code_id,
            chain_id,
            msg: init_msg,
        }),
        key,
        creation_fee(&chain),
    )
}



pub fn migrate_simple_token_account<C: CosmosClient>(
    chain: &mut Chain<C>,
    token_contract: String,
    token_id: String,
    key: &SigningKey,
) -> Result<ExecResponse, ProcessError> {
    let code_id = chain.orc.contract_map.code_id(SIMPLE_ACCOUNT_NAME)?;

    let migrate_msg = cw83_tba_registry::msg::ExecuteMsg::MigrateAccount {
        token_info: TokenInfo {
            collection: token_contract,
            id: token_id,
        },
        new_code_id: code_id,
        msg: MigrateAccountMsg {
            params: None,
        },
    };

    chain.orc.execute(
        BASE_REGISTRY_NAME,
        "registry_reset_account",
        &migrate_msg,
        key,
        vec![],
    )
}

#[cw_serde]
pub struct FullSetupData {
    pub registry: String,
    pub collection: String,

    pub token_id: String,
    pub token_account: String,

    pub cred_token_id: String,
    pub cred_token_account: String,

    pub signer_address: String,
    pub signer_mnemonic: String,

    pub private_key: SigningKey,
    pub public_key: Binary,
}

pub fn get_init_address(res: ChainTxResponse) -> String {
    res.find_event_tags("instantiate".to_string(), "_contract_address".to_string())[0]
        .value
        .clone()
}

pub fn full_setup<C: CosmosClient>(chain: &mut Chain<C>) -> Result<FullSetupData, ProcessError> {
    
    let _start_time = latest_block_time(chain).plus_seconds(60);

    let user: super::chain::SigningAccount = chain.cfg.users[0].clone();
    let signer_address = user.account.address.clone();
    let signer_mnemonic = user.account.mnemonic.clone();

    let reg_init = instantiate_registry(chain, signer_address.clone(), &user.key).unwrap();
    let registry = get_init_address(reg_init.res);


    let init_res =
        instantiate_collection(chain, user.account.address.clone(), None, &user.key).unwrap();


    let collection = get_init_address(init_res.res);
    chain
        .orc
        .contract_map
        .add_address(COLLECTION_NAME, collection.clone())
        .unwrap();

    let token_id = "1".to_string();
    mint_token(
        chain,
        token_id.clone(),
        user.account.address.clone(),
        &user.key,
    )
    .unwrap();

    let pubkey: Binary = user.key.public_key().unwrap().to_bytes().into();
    let create_res = create_simple_token_account(
        chain,
        collection.clone(),
        token_id.clone(),
        pubkey.clone(),
        &user.key,
    )
    .unwrap();

    let token_account = get_init_address(create_res.res);
    chain
        .orc
        .contract_map
        .add_address(SIMPLE_ACCOUNT_NAME, token_account.clone())
        .unwrap();


    let cred_token_id = "2".to_string();
    mint_token(
        chain,
        cred_token_id.clone(),
        user.account.address.clone(),
        &user.key,
    )
    .unwrap();

    let cred_create_res = create_cred_token_account(
        chain,
        collection.clone(),
        cred_token_id.clone(),
        &user
    );

    let cred_token_account = get_init_address(cred_create_res.unwrap().res);
    chain
        .orc
        .contract_map
        .add_address(CRED_ACOUNT_NAME, cred_token_account.clone())
        .unwrap();

    Ok(FullSetupData {
        registry,
        collection,

        token_id,
        token_account,

        cred_token_id,
        cred_token_account,

        signer_address,
        signer_mnemonic,

        private_key: user.key,
        public_key: pubkey,
    })
}

pub fn wasm_query<C: CosmosClient, S: Serialize>(
    chain: &mut Chain<C>,
    address: &String,
    msg: &S,
) -> Result<QueryResponse, CosmwasmError> {
    let res = tokio_block(async {
        chain
            .orc
            .client
            .wasm_query(Address::from_str(&address)?, msg)
            .await
    });

    res
}

pub fn wasm_query_typed<C, R, S>(
    chain: &mut Chain<C>,
    address: &String,
    msg: &S,
) -> Result<R, CosmwasmError>
where
    C: CosmosClient,
    S: Serialize,
    R: DeserializeOwned,
{
    let res = tokio_block(async {
        chain
            .orc
            .client
            .wasm_query(Address::from_str(&address)?, msg)
            .await
    })?;

    let res: R = from_json(&res.res.data.unwrap()).unwrap();

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
            token_id,
            include_expired: None,
        },
    )
    .unwrap();

    let owner_res: cw721::OwnerOfResponse = from_json(&res.res.data.unwrap()).unwrap();

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
        users.push(SigningKey::random_mnemonic(
            n.to_string(),
            cfg.derivation_path.clone(),
        ));
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
    msg: CosmosMsg,
) -> CanExecuteResponse {
    let res = wasm_query(
        chain,
        token_account,
        &QueryMsg::CanExecute {
            sender: sender,
            msg: msg.into(),
        },
    )
    .unwrap();

    from_json(&res.res.data.unwrap()).unwrap()
}
