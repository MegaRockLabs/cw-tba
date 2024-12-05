// allow unreachable code for testing
#![allow(unreachable_code, unused_mut, unused_imports)]

#[cfg(test)]
mod tests {
    

    use cosmwasm_std::{
        coins, from_json, testing::{mock_dependencies, mock_env, mock_info}, to_json_binary, to_json_string, Addr, Coin, CosmosMsg, MessageInfo, StakingMsg, Uint128
    };
    use cw_tba::{encode_feegrant_msg, BasicAllowance, ExecuteAccountMsg, TokenInfo};
    use saa::{messages::{MsgDataToSign, MsgDataToVerify, SignedDataMsg}, Binary, CosmosArbitrary, Credential, CredentialData, PasskeyCredential, Verifiable};

    use crate::{
        contract::{execute, instantiate}, msg::{ExecuteMsg, InstantiateMsg}, 
    };


    #[test]
    fn can_verify_plaintext() {

        let mut deps = mock_dependencies();
        

        let cred = Credential::CosmosArbitrary(saa::CosmosArbitrary {
            pubkey: Binary::from_base64("A2LjUH7Q0gi7+Wi0/MnXMZqN8slsz7iHMfTWp8xUXspH").unwrap(),
            signature: Binary::from_base64("TFcYDwzxeRLqowzTOCx0RL0pvDgKngh8ijdNBzFEcMtu5HZVhN03sY3BG9DNIqwuuiJkZDcQFE2CCVM5PwLHpQ==").unwrap(),
            message: Binary::new("Hello".as_bytes().to_vec()),
            hrp: Some(String::from("archway")),
        });
        let res = cred.verify_cosmwasm(deps.as_ref().api);
        assert!(res.is_ok());

        let auth_data  = CredentialData {
            credentials: vec![cred],       
            with_caller: None,
            primary_index: None,
        };

        let res = auth_data.verify_cosmwasm(deps.as_ref().api);
        println!("{:?}", res);
        assert!(res.is_ok());
    }


    

    #[test]
    fn custom_cosmos_msg_verifiable() {
        
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.chain_id = "elgafar-1".to_string();

        let public_key = Binary::from_base64("BOirsl/nNsTWj3O5Qfseo9qZfs0uakJ6I97JLDZSbmeYk6nwkjIHM7UKp1DD/UnmurwUMFoqRIkO7sqsRFg8eUU=").unwrap();
        let auth_data  = Binary::from_base64("SZYN5YgOjGh0NBcPZHZgW4/krrmihjLHmVzzuoMdl2MdAAAAAA==").unwrap();
        let signature = Binary::from_base64("xEQi+PS2JWc/VFEXWBkbZW4A7WG5iAwIgjGlrqtKVqbqrrnGfAqMSyHxEUOWBWspa4F/U+itj3cWQUcaaBAXHA==").unwrap();

        let credential = Credential::Passkey(PasskeyCredential { 
            id: String::default(),
            pubkey: Some(public_key), 
            signature, 
            authenticator_data: auth_data, 
            client_data: saa::ClientData {
                ty: "webauthn.get".to_string(),
                challenge: "eyJjaGFpbl9pZCI6ImVsZ2FmYXItMSIsImNvbnRyYWN0X2FkZHJlc3MiOiJzdGFyczFjbGE0cmFudnVkZWRrMDM2aGV1cGYyanVwcnhoYWFxY2RkZmtmbDZqbnBjbTh1NWQwNXpxbHVna2V0IiwibWVzc2FnZXMiOlsiQ3JlYXRlIFRCQSBhY2NvdW50Il0sIm5vbmNlIjoiMCJ9".to_string(),
                cross_origin: false,
                origin: "http://localhost:5173".into(),
            }, 
            user_handle: None
        });


        let info = MessageInfo {
            sender: Addr::unchecked("stars1cla4ranvudedk036heupf2juprxhaaqcddfkfl6jnpcm8u5d05zqlugket"),
            funds: vec![]
        };

        let init_res = instantiate(
            deps.as_mut(), 
            env.clone(), 
            info.clone(), 
            InstantiateMsg {
                owner: String::from("owner"),
                account_data: to_json_binary(&CredentialData {
                    credentials: vec![credential],
                    with_caller: Some(true),
                    primary_index: None,
                }).unwrap(),
                token_info: TokenInfo {
                    collection: "test".into(),
                    id: "test".into()
                },
                actions: None
            }
        );
        
        println!("Init res: {:?}", init_res);
        assert!(init_res.is_ok());


      /*   let deps = deps.as_mut();

        let custom = SignedDataMsg {
            data: to_json_binary(&data).unwrap().into(),
            signature: signature.clone().into(),
            payload: None
        };

        let msg = CosmosMsg::<SignedDataMsg>::Custom(custom);

        let execute_msg = ExecuteMsg::Execute { 
            msgs: vec![msg]
        };

        let res = execute(
            deps, 
            env.clone(), 
            info,
            execute_msg
        );
 */
        
    }
 

    #[test]
    fn can_check_passkeys() {

        let mut deps = mock_dependencies();
        let deps = deps.as_ref();

        let public_key = Binary::from_base64("BOirsl/nNsTWj3O5Qfseo9qZfs0uakJ6I97JLDZSbmeYk6nwkjIHM7UKp1DD/UnmurwUMFoqRIkO7sqsRFg8eUU=").unwrap();
        let auth_data  = Binary::from_base64("SZYN5YgOjGh0NBcPZHZgW4/krrmihjLHmVzzuoMdl2MdAAAAAA==").unwrap();
        let signature = Binary::from_base64("12hV75Rn674rsrgvLsvFox97B69KURvB4NtdJI7zIdrfq6EZBc2a7ZbUp1R/ktHPk8R4tNXV1YZQ56lojaR2jg==").unwrap();

        let credential = Credential::Passkey(PasskeyCredential { 
            id: String::default(),
            pubkey: Some(public_key), 
            signature, 
            authenticator_data: auth_data, 
            client_data: saa::ClientData {
                ty: "webauthn.get".to_string(),
                challenge: "eyJjaGFpbl9pZCI6ImVsZ2FmYXItMSIsImNvbnRyYWN0X2FkZHJlc3MiOiJzdGFyczFjbGE0cmFudnVkZWRrMDM2aGV1cGYyanVwcnhoYWFxY2RkZmtmbDZqbnBjbTh1NWQwNXpxbHVna2V0IiwibWVzc2FnZXMiOlsiQ3JlYXRlIFRCQSBhY2NvdW50Il0sIm5vbmNlIjoiMCJ9".to_string(),
                cross_origin: false,
                origin: "http://localhost:5173".into(),
            }, 
            user_handle: None
        });

        let res = credential.verify_cosmwasm(deps.api);
        println!("{:?}", res);
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
            CosmosMsg::Stargate { type_url, value} =>  {
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
