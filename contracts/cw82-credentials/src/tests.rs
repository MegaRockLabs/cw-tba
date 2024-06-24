#[cfg(test)]
mod tests {
    
    use cosmwasm_std::{testing::{mock_dependencies, mock_env, mock_info}, to_json_binary};
    use cw22::set_contract_supported_interface;
    use cw_tba::TokenInfo;
    use saa::{Binary, Credential, CredentialData, Verifiable};

    use crate::{contract::instantiate, msg::InstantiateMsg};

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
        println!("self auth {:?}", res);
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

        println!("inst auth {:?}", res);
        assert!(res.is_ok())
        
    }
}
