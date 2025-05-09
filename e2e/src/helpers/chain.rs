use cosm_orc::config::error::ConfigError;
use cosm_orc::config::ChainConfig;
use cosm_orc::orchestrator::{CosmosgRPC, Key, SigningKey, TendermintRPC};
use cosm_orc::{config::cfg::Config, orchestrator::cosm_orc::CosmOrc};
use cosm_tome::clients::client::CosmosClient;
use once_cell::sync::OnceCell;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::fs;
use std::path::Path;
use std::time::Duration;
use test_context::TestContext;

static CONFIG: OnceCell<Cfg> = OnceCell::new();

#[derive(Clone, Debug)]
pub struct Cfg {
    pub orc_cfg: Config,
    pub users: Vec<SigningAccount>,
    pub gas_report_dir: String,
}

#[derive(Clone, Debug)]
pub struct SigningAccount {
    pub account: Account,
    pub key: SigningKey,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Account {
    pub name: String,
    pub address: String,
    pub mnemonic: String,
}

#[derive(Clone, Debug)]
pub struct Chain<C = TendermintRPC>
where
    C: CosmosClient,
{
    pub cfg: Cfg,
    pub orc: CosmOrc<C>,
}


impl TestContext for Chain<CosmosgRPC> {
    fn setup() -> Self {
        let cfg = CONFIG.get_or_init(|| {
            let orc_cfg = init_orc_config().unwrap();
            let orc = CosmOrc::new(orc_cfg.clone(), true).unwrap();
            global_setup(orc, orc_cfg)
        });

        let orc = CosmOrc::new(cfg.orc_cfg.clone(), true).unwrap();

        Self {
            cfg: cfg.clone(),
            orc,
        }
    }

    fn teardown(self) {
        let cfg = CONFIG.get().unwrap();
        save_gas_report(&self.orc, &cfg.gas_report_dir);
    }
}

impl TestContext for Chain<TendermintRPC> {
    fn setup() -> Self {
        let cfg = CONFIG.get_or_init(|| {
            let orc_cfg = init_orc_config().unwrap();
            let orc = CosmOrc::new_tendermint_rpc(orc_cfg.clone(), true).unwrap();
            global_setup(orc, orc_cfg)
        });

        let orc = CosmOrc::new_tendermint_rpc(cfg.orc_cfg.clone(), true).unwrap();

        Self {
            cfg: cfg.clone(),
            orc,
        }
    }

    fn teardown(self) {
        let cfg = CONFIG.get().unwrap();
        save_gas_report(&self.orc, &cfg.gas_report_dir);
    }
}

fn init_orc_config() -> Result<Config, ConfigError> {
    let mut config = env::var("CONFIG");
    if config.is_err() {
        config = Ok("configs/cosm-orc.yaml".to_string());
    }
    let config = config.expect("missing yaml CONFIG env var");
    Config::from_yaml(&config)
}

// global_setup() runs once before all of the tests:
// - loads cosm orc / test account config files
// - stores contracts on chain for all tests to reuse
fn global_setup<C: CosmosClient>(mut orc: CosmOrc<C>, mut orc_cfg: Config) -> Cfg {
    env_logger::init();

    let gas_report_dir = env::var("GAS_OUT_DIR").unwrap_or_else(|_| "gas_reports".to_string());
    let accounts = test_accounts(&orc_cfg.chain_cfg);

    // Poll for first block to make sure the node is up:
    orc.poll_for_n_blocks(1, Duration::from_millis(10_000), true)
        .unwrap();

    let skip_storage = env::var("SKIP_CONTRACT_STORE").unwrap_or_else(|_| "false".to_string());
    if !skip_storage.parse::<bool>().unwrap() {
        orc.store_contracts("../artifacts", &accounts[0].key, None)
            .unwrap();

        save_gas_report(&orc, &gas_report_dir);

        // persist stored code_ids in CONFIG, so we can reuse for all tests
        orc_cfg.contract_deploy_info = orc.contract_map.deploy_info().clone();

        // println!("Contract deploy info: {:?}", orc_cfg.contract_deploy_info);
    }

    Cfg {
        orc_cfg,
        users: accounts,
        gas_report_dir,
    }
}

fn test_accounts(cfg: &ChainConfig) -> Vec<SigningAccount> {
    let bytes = fs::read("configs/test_accounts.json").unwrap();
    let accounts: Vec<Account> = serde_json::from_slice(&bytes).unwrap();

    accounts
        .into_iter()
        .map(|a| SigningAccount {
            account: a.clone(),
            key: SigningKey {
                name: a.name,
                key: Key::Mnemonic(a.mnemonic),
                derivation_path: cfg.derivation_path.clone(),
            },
        })
        .collect()
}

fn save_gas_report<C: CosmosClient>(orc: &CosmOrc<C>, gas_report_dir: &str) {
    let report = orc
        .gas_profiler_report()
        .expect("error fetching profile reports");

    let j: Value = serde_json::to_value(report).unwrap();

    let p = Path::new(gas_report_dir);
    if !p.exists() {
        fs::create_dir(p).unwrap();
    }

    let mut rng = rand::thread_rng();
    let file_name = format!("test-{}.json", rng.gen::<u32>());
    fs::write(p.join(file_name), j.to_string()).unwrap();
}
