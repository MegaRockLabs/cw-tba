// allow unreachable code for testing
#![allow(unreachable_code, unused_mut, unused_imports)]

#[cfg(test)]
mod tests {
    

    use cosmwasm_std::{
        coins, testing::{mock_dependencies, mock_env, mock_info}, to_json_binary, to_json_string, Addr, Coin, CosmosMsg, MessageInfo, StakingMsg, Uint128
    };
    use cw_tba::{encode_feegrant_msg, BasicAllowance, ExecuteAccountMsg, TokenInfo};
    use saa::{messages::{MsgDataToSign, SignedData}, Binary, CosmosArbitrary, Credential, CredentialData, PasskeyCredential, Verifiable};

    use crate::{
        contract::{execute, instantiate}, msg::{ExecuteMsg, InstantiateMsg}, 
    };


    #[test]
    fn can_instantiate_plaintext() {

        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("alice", &vec![]);
        

        let cred = Credential::CosmosArbitrary(saa::CosmosArbitrary {
            pubkey: Binary::from_base64("A2LjUH7Q0gi7+Wi0/MnXMZqN8slsz7iHMfTWp8xUXspH").unwrap(),
            signature: Binary::from_base64("TFcYDwzxeRLqowzTOCx0RL0pvDgKngh8ijdNBzFEcMtu5HZVhN03sY3BG9DNIqwuuiJkZDcQFE2CCVM5PwLHpQ==").unwrap(),
            message: Binary("Hello".as_bytes().to_vec()),
            hrp: Some(String::from("archway")),
        });
        let res = cred.verify_cosmwasm(deps.as_ref().api, &env);
        assert!(res.is_ok());


        let auth_data  = CredentialData {
            credentials: vec![cred],       
            with_caller: Some(true),
            primary_index: None,
        };

        let res = auth_data.verify_cosmwasm(deps.as_ref().api, &env);
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
                actions: None
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


        let res = auth_data.verify_cosmwasm(deps.as_ref().api, &env);
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
                actions: None
            }
        );

        assert!(res.is_ok())
    }


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

        let data = MsgDataToSign {
            chain_id: "constantine-3".into(),
            contract_address: "archway1hf0quw8lgxn4p9vlmk3jdlgg460asp87c75s9xfm33axkczu2j3s7mwfke".into(),
            messages: vec![action],
            nonce: String::from("0"),
        };

        let pubkey = Binary::from_base64(
            "A2LjUH7Q0gi7+Wi0/MnXMZqN8slsz7iHMfTWp8xUXspH"
        ).unwrap();

        let signature = Binary::from_base64(
            "EfGD3KMZUMppuA5+3AQ2xQPblr4FQpVWyZi/9+Vry0MVGWhJqeECPuwIkhEgaeTL6tFrOIEkYAY1I7L7uz9+Fg=="
        ).unwrap();


        let message = SignedData::<ExecuteAccountMsg> {
            data: data.clone(),
            signature: signature.clone(),
            payload: None
        };

        let cred  =  CosmosArbitrary  {
            message: to_json_binary(&message).unwrap().into(),
            signature: signature.clone().into(),
            pubkey: pubkey.clone().into(),
            hrp: Some("archway".to_string())
        };
        let res = cred.verify_cosmwasm(deps.as_ref().api, &env);
        

        assert!(res.is_ok());


        let custom = SignedData::<ExecuteAccountMsg> {
            data: data.clone(),
            signature: signature.clone().into(),
            payload: None
        };

        let msg = CosmosMsg::
                <SignedData::<ExecuteAccountMsg>>::Custom(custom);

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
                actions: None
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
 

    #[test]
    fn can_check_passkeys() {

        let mut deps = mock_dependencies();
        let deps = deps.as_ref();
        let env = mock_env();

        let public_key = Binary::from_base64("BGDRdC9Ynea9vlpLxFZmEGL1cYpxGgzRvEMzlugVfmYOyACjQ5wHA8DNuCR4GI/Sfj6OkVNlyvuwyfkeOPavcG8=").unwrap();
        let auth_data  = Binary::from_base64("SZYN5YgOjGh0NBcPZHZgW4/krrmihjLHmVzzuoMdl2MFAAAAAA==").unwrap();
        let signature = Binary::from_base64("6dMQf0mPwkFBPuAElErBTi3SbqhWKRVxZVix/YwcbxxPLEGifo+KITlWmY4CX/ZoVJllFmW03DYKWwNo+7lIOw==").unwrap();

        let credential = Credential::Passkey(PasskeyCredential { 
            id: String::default(),
            pubkey: Some(public_key), 
            signature, 
            authenticator_data: auth_data, 
            client_data: saa::ClientData {
                ty: "webauthn.get".to_string(),
                challenge: Binary::from_base64("Q3JlYXRpbmcgVEJBIGFjY291bnQ").unwrap(),
                cross_origin: false,
                origin: "http://localhost:5173".into(),
            }, 
            user_handle: None
        });

        let res = credential.verify_cosmwasm(deps.api, &env);

        println!("Res: {:?}", res);
        assert!(res.is_ok());
    }

    #[test]
    fn fee_grant_msg() {

        let allowance = Some(BasicAllowance {
            spend_limit: coins(1000000, "ustars"),
            expiration: None,
        });

        let msg = encode_feegrant_msg(
            "stars1shqqdheghk6reu525whq0cav0d43t3auemx7ayanwmtm742egxes9h2kc2", 
            "stars1v85m4sxnndwmswtd8jrz3cd2m8u8eegqdxyluz", 
            allowance
        ).unwrap();

        match msg {
            CosmosMsg::Stargate { type_url, value } => {
                assert!(type_url == "/cosmos.feegrant.v1beta1.MsgGrantAllowance");
                assert_eq!(
                    value.to_base64().as_str(),
                    "CkBzdGFyczFzaHFxZGhlZ2hrNnJldTUyNXdocTBjYXYwZDQzdDNhdWVteDdheWFud210bTc0MmVneGVzOWgya2MyEixzdGFyczF2ODVtNHN4bm5kd21zd3RkOGpyejNjZDJtOHU4ZWVncWR4eWx1ehqWAQosL2Nvc21vcy5mZWVncmFudC52MWJldGExLkFsbG93ZWRNc2dBbGxvd2FuY2USZgo+CicvY29zbW9zLmZlZWdyYW50LnYxYmV0YTEuQmFzaWNBbGxvd2FuY2USEwoRCgZ1c3RhcnMSBzEwMDAwMDASJC9jb3Ntd2FzbS53YXNtLnYxLk1zZ0V4ZWN1dGVDb250cmFjdA=="
                )
            },
            _ => {
                unreachable!()
            }
        }
    }


}