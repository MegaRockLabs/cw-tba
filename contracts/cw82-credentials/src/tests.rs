#[cfg(test)]
mod tests {
  


    #[test]
    fn amino_check() {

       
    }


    /* #[test]
    fn amino_check_contract() {

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
        

        instantiate(
            deps.as_mut(), 
            env.clone(), 
            info.clone(), 
            InstantiateMsg {
                owner: ACCOUNT.into(),
                account_data: Binary::from_base64(PUBKEY).unwrap(),
                token_info: TokenInfo {
                    collection: "test".into(),
                    id: "test".into()
                },
            }
        ).unwrap();

        let msg = QueryMsg::ValidSignature { 
            data: to_json_binary(&MSG).unwrap(), 
            signature: Binary::from_base64(SIGNATURE).unwrap(), 
            payload: None
        };

        let query_res = query(
            deps.as_ref(), 
            env.clone(), 
            msg
        ).unwrap();

        let res : ValidSignatureResponse = from_json(&query_res).unwrap();

        assert!(res.is_valid)

        
    }
    */

}