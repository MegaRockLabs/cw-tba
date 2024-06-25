#[cfg(test)]
mod tests {
    
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info}, to_json_binary, to_json_string, Addr, Coin, CosmosMsg, MessageInfo, StakingMsg, Uint128
    };
    use cw22::set_contract_supported_interface;
    use cw_tba::TokenInfo;
    use saa::{Binary, CosmosArbitrary, Credential, CredentialData, Verifiable};

    use crate::{contract::{execute, instantiate}, msg::{CosmosMsgDataToSign, ExecuteMsg, InstantiateMsg, SignedCosmosMsgs}};

    const MESSAGE : &str = "SGVsbG8sIHdvcmxk";
    const SIGNATURE : &str = "x9jjSFv8/n1F8gOSRjddakYDbvroQm8ZoDWht/Imc1t5xUW49+Xaq7gwcsE+LCpqYoTBxnaXLg/xgJjYymCWvw==";
    const PUBKEY : &str = "A08EGB7ro1ORuFhjOnZcSgwYlpe0DSFjVNUIkNNQxwKQ";
    const HRP : &str = "cosmos";

    
    #[test]
    fn can_instantiate() {

        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("alice", &vec![]);

        set_contract_supported_interface(
            deps.as_mut().storage,
            &[cw22::ContractSupportedInterface {
                supported_interface: cw83::INTERFACE_NAME.into(),
                version: "0.0.0".into()
            }]
        ).unwrap();


         let cred = Credential::CosmosArbitrary(saa::CosmosArbitrary {
            pubkey: Binary::from_base64(PUBKEY).unwrap(),
            signature: Binary::from_base64(SIGNATURE).unwrap(),
            message: Binary::from_base64(MESSAGE).unwrap(),
            hrp: Some(HRP.into()),
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


    #[test]
    fn custom_cosmos_msg_verifiable() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let staking = StakingMsg::Delegate { 
            amount: Coin { 
                amount: Uint128::from(1000000000000000000u128),
                denom: "aconst".into()
            }, 
            validator: "archwayvaloper1qt0e4eyswes6qpply2pmk8v5qm88r2c962fnvk".into(), 
        };

        let data = CosmosMsgDataToSign {
            chain_id: "constantine-3".into(),
            messages: vec![staking.into()],
            nonce: Uint128::zero()
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
        assert!(res.is_ok());


        let custom = SignedCosmosMsgs {
            data: data.clone(),
            signature: signature.clone().into(),
            payload: None
        };

        let msg = CosmosMsg::<SignedCosmosMsgs>::Custom(custom);

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


}
