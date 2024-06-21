use cosm_tome::signing_key::key::mnemonic_to_signing_key;
use cosmwasm_std::{to_json_binary, to_json_string, Empty, Uint128};
use cw_tba::ExecuteAccountMsg;
use saa::cosmos_utils::preamble_msg_arb_036;
use test_context::test_context;

use cw82_credentials::msg::{AccountActionDataToSign, ExecuteMsg, SignedAccountActions};
use crate::helpers::helper::{
    get_init_address, instantiate_collection, CRED_ACOUNT_NAME
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

    
    let execute_msg = ExecuteAccountMsg::MintToken {
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
    println!("{:?}", res);
    res.unwrap_err();



    let actions = AccountActionDataToSign { 
        chain_id: chain.cfg.orc_cfg.chain_cfg.chain_id.clone(),
        nonce: Uint128::from(1u64),
        actions: vec![execute_msg]
    };

    let sk = mnemonic_to_signing_key(
        &user.account.mnemonic, 
        &user.key.derivation_path
    ).unwrap();


    let signature = sk.sign(
        &preamble_msg_arb_036(
            &user.account.address, 
            &to_json_string(&actions).unwrap()
        ).as_bytes()
    ).unwrap();

    let signed = SignedAccountActions { 
        data: actions, 
        signature: signature.to_vec().into(),
        payload: None, 
    };

    let msg = ExecuteMsg::Extension { 
        msg: signed.into(),
    };



    let res = chain
        .orc
        .execute(
            CRED_ACOUNT_NAME,
            "cred_tokens_mint",
            &msg,
            &user.key,
            vec![],
        );

    println!("{:?}", res);
    assert!(res.is_ok())

}



