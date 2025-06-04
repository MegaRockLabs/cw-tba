use cosm_tome::signing_key::key::mnemonic_to_signing_key;
use cosmwasm_std::testing::{mock_info, mock_dependencies, mock_env};
use cosmwasm_std::{to_json_binary, Addr, CosmosMsg, Empty};
use cw82_tba_credentials::contract::instantiate;
use cw82_tba_credentials::execute::try_executing;
use cw_tba::{ExecuteMsg, TokenInfo};
use saa_wasm::saa_types::utils::cosmos::preamble_msg_arb_036;
use saa_wasm::saa_types::msgs::{MsgDataToSign, SignedDataMsg};
use test_context::test_context;

use cw82_tba_credentials::msg::InstantiateMsg;

use crate::helpers::helper::{
    get_cred_data, get_init_address, instantiate_collection, CRED_ACOUNT_NAME
};

use crate::helpers::{
    chain::Chain,
    helper::full_setup,
};


#[test_context(Chain)]
#[test]
#[ignore]
fn test(chain: &mut Chain) {
    let data = full_setup(chain).unwrap();
    let user = chain.cfg.users[0].clone();

    let init_res =
        instantiate_collection(chain, data.cred_token_account.clone(), None, &user.key).unwrap();

    let collection = get_init_address(init_res.res);

    let mint_msg: cw721_base::ExecuteMsg<Option<Empty>, Empty> = cw721_base::ExecuteMsg::Mint {
        token_id: "1".to_string(),
        owner: data.cred_token_account.clone(),
        token_uri: None,
        extension: None,
    };

    
    let execute_msg = ExecuteMsg::MintToken {
        minter: collection.clone(),
        msg: to_json_binary(&mint_msg).unwrap(),
    };

        
    let res = chain
        .orc
        .execute(
            CRED_ACOUNT_NAME,
            "cred_tokens_mint_403",
            &execute_msg,
            &user.key,
            vec![],
        );
    // not authorized to call direcly
    res.unwrap_err();


    let actions = MsgDataToSign::<ExecuteMsg> { 
        chain_id: chain.cfg.orc_cfg.chain_cfg.chain_id.clone(),
        contract_address: data.cred_token_account.clone(),
        messages: vec![execute_msg],
        nonce: 1u64.into(),
    };

    let sk = mnemonic_to_signing_key(
        &user.account.mnemonic, 
        &user.key.derivation_path
    ).unwrap();


    let signature = sk.sign(
        &preamble_msg_arb_036(
            &user.account.address, 
            &to_json_binary(&actions).unwrap().to_base64()
        ).as_bytes()
    ).unwrap();


    let signed = SignedDataMsg { 
        data: to_json_binary(&actions).unwrap().into(), 
        signature: signature.to_vec().into(),
        payload: None
    };

    let msgs = vec![CosmosMsg::Custom(signed)];

    let msg = ExecuteMsg::Execute { 
        msgs: msgs.clone()
    };

    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let info = mock_info(user.account.address.as_str(), &vec![]);

    env.block.chain_id = chain.cfg.orc_cfg.chain_cfg.chain_id.clone();
    env.contract.address = Addr::unchecked(data.cred_token_account.clone());

    println!("contract address: {:?}", env.contract.address);

    let account_data = get_cred_data(chain, &user, msgs.clone());

    let init_msg = InstantiateMsg {
        owner: user.account.address.clone(),
        account_data,
        token_info: TokenInfo {
            collection: data.collection.clone(),
            id: data.token_id.clone()
        },
        actions: None
    };

    println!("Registry: {:?}", data.registry);

    println!("User account: {:?}", user.account.address);

    let init_res = instantiate(
        deps.as_mut(), 
        env.clone(), 
        mock_info(data.registry.as_str(), &[]), 
        init_msg
    );

    println!("Init res: {:?}", init_res);
    assert!(init_res.is_ok());
    init_res.unwrap();


    let exec_res = try_executing(
        deps.as_mut(),
        env.clone(),
        info,
        msgs,
    );
    assert!(exec_res.is_ok());

    let res = chain
        .orc
        .execute(
            CRED_ACOUNT_NAME,
            "cred_tokens_mint",
            &msg,
            &user.key,
            vec![],
        );

    assert!(res.is_ok())

}



