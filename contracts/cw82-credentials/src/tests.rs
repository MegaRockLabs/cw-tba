#[cfg(test)]
mod tests {
    
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info}, to_json_binary
    };
    use cw_tba::TokenInfo;
    use saa::{Binary, Credential, CredentialData, Verifiable};

    use crate::{contract::instantiate, msg::InstantiateMsg};



    #[test]
    fn can_instantiate_plaintext() {

        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("alice", &vec![]);


        let cred = Credential::CosmosArbitrary(saa::CosmosArbitrary {
            pubkey: Binary::from_base64("A2LjUH7Q0gi7+Wi0/MnXMZqN8slsz7iHMfTWp8xUXspH").unwrap(),
            signature: Binary::from_base64("TFcYDwzxeRLqowzTOCx0RL0pvDgKngh8ijdNBzFEcMtu5HZVhN03sY3BG9DNIqwuuiJkZDcQFE2CCVM5PwLHpQ==").unwrap(),
            message: Binary("Hello".as_bytes().to_vec()).to_base64(),
            hrp: Some(String::from("archway")),
        });
        let res = cred.verified_cosmwasm(deps.as_ref().api, &env, &None);
        assert!(res.is_ok());


        let auth_data  = CredentialData {
            credentials: vec![cred],       
            with_caller: Some(true),
            primary_index: None,
        };

        let res = auth_data.verified_cosmwasm(deps.as_ref().api, &env, &None);
        assert!(res.is_ok());

        let res = instantiate(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            InstantiateMsg {
                owner: info.sender.into(),
                account_data: to_json_binary(&auth_data).unwrap(),
                token_info: TokenInfo {
                    collection: "test".into(),
                    id: "test".into()
                },
            }
        );

        assert!(res.is_ok())
    }



    
    #[test]
    fn can_instantiate_base64() {

        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("alice", &vec![]);


         let cred = Credential::CosmosArbitrary(saa::CosmosArbitrary {
            pubkey: Binary::from_base64("A08EGB7ro1ORuFhjOnZcSgwYlpe0DSFjVNUIkNNQxwKQ").unwrap(),
            signature: Binary::from_base64("x9jjSFv8/n1F8gOSRjddakYDbvroQm8ZoDWht/Imc1t5xUW49+Xaq7gwcsE+LCpqYoTBxnaXLg/xgJjYymCWvw==").unwrap(),
            message: Binary::from_base64("SGVsbG8sIHdvcmxk").unwrap(),
            hrp: Some(String::from("cosmos")),
        });


        let auth_data  = CredentialData {
            credentials: vec![cred],       
            with_caller: Some(true),
            primary_index: None,
        };


        let res = auth_data.verified_cosmwasm(deps.as_ref().api, &env, &None);
        assert!(res.is_ok());

        let res = instantiate(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            InstantiateMsg {
                owner: info.sender.into(),
                account_data: to_json_binary(&auth_data).unwrap(),
                token_info: TokenInfo {
                    collection: "test".into(),
                    id: "test".into()
                },
            }
        );

        assert!(res.is_ok())
    }

    /*
    #[test]
    fn custom_cosmos_msg_verifiable() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let staking : CosmosMsg = StakingMsg::Delegate { 
            amount: Coin { 
                amount: Uint128::from(1000000000000000000u128),
                denom: "aconst".into()
            }, 
            validator: "archwayvaloper1qt0e4eyswes6qpply2pmk8v5qm88r2c962fnvk".into(), 
        }.into();

        let action = ExecuteAccountMsg::Execute { 
            msgs: vec![staking] 
        };

        let data = ActionDataToSign {
            chain_id: "constantine-3".into(),
            messages: vec![action],
            nonce: Uint128::zero(),
        };

        let pubkey = Binary::from_base64(
            "A2LjUH7Q0gi7+Wi0/MnXMZqN8slsz7iHMfTWp8xUXspH"
        ).unwrap();

        let signature = Binary::from_base64(
            "QmL2kedji/xGtcHHAf6eMz0xk0yF6wGIpVyIE18gEld96skk4tkplmirmd/2+oh+hMvVtenNlBycmuTxb22XBw=="
        ).unwrap();

        let data_str = to_json_string(&data).unwrap();


        let message = Binary(data_str.as_bytes().to_vec());

        let cred  =  CosmosArbitrary  {
            message: message.clone(),
            signature: signature.clone(),
            pubkey: pubkey.clone(),
            hrp: Some("archway".to_string())
        };
        let res = cred.verified_cosmwasm(deps.as_ref().api, &env, &None);
        
        println!("Execute res: {:?}", res);

        
        assert!(res.is_ok());


        let custom = SignedActions {
            data: data.clone(),
            signature: signature.clone().into(),
            payload: None
        };

        let msg = CosmosMsg::<SignedActions>::Custom(custom);

        let execute_msg = ExecuteMsg::Execute { 
            msgs: vec![msg]
        };

        let owner = String::from("archway1v85m4sxnndwmswtd8jrz3cd2m8u8eegqv30xay");

        let info = MessageInfo {
            sender: Addr::unchecked("registry"),
            funds: vec![]
        };

        instantiate(
            deps.as_mut(), 
            env.clone(), 
            info.clone(), 
            InstantiateMsg {
                owner,
                account_data: to_json_binary(&CredentialData {
                    credentials: vec![Credential::CosmosArbitrary(cred)],
                    with_caller: None,
                    primary_index: None,
                }).unwrap(),
                token_info: TokenInfo {
                    collection: "test".into(),
                    id: "test".into()
                },
            }
        ).unwrap();


        let deps = deps.as_mut();

        let res = execute(
            deps, 
            env.clone(), 
            info,
            execute_msg
        );

        
        assert!(res.is_ok())
    }
 */


}
