use crate::{
    contract::{instantiate, query},
    msg::{InstantiateMsg, QueryMsg},
    query::verify_arbitrary,
};
use cosmwasm_std::{
    from_json,
    testing::{mock_dependencies, mock_env, mock_info},
    to_json_binary, Binary,
};
use cw22::set_contract_supported_interface;
use cw82::ValidSignatureResponse;
use cw_tba::TokenInfo;

const MSG: &str = "dGVzdA==";
const PUBKEY: &str = "A2LjUH7Q0gi7+Wi0/MnXMZqN8slsz7iHMfTWp8xUXspH";
const SIGNATURE: &str =
    "6UDr+Cu5+6SAgbMRj3hQfXZecdpxsmznLfTMcWkXPDl1DBJRNg+XrFal3BqF8TWJ+o9KM8+z5sfZZ1hfUPkSbg==";
const ACCOUNT: &str = "stars1v85m4sxnndwmswtd8jrz3cd2m8u8eegqdxyluz";

#[test]
fn amino_check() {
    let deps = mock_dependencies();

    let ok = verify_arbitrary(
        deps.as_ref(),
        ACCOUNT,
        to_json_binary(MSG).unwrap(),
        Binary::from_base64(SIGNATURE).unwrap(),
        Binary::from_base64(PUBKEY).unwrap().as_slice(),
    )
    .unwrap();
    assert!(ok);
}

#[test]
fn amino_check_contract() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("alice", &[]);

    set_contract_supported_interface(
        deps.as_mut().storage,
        &[cw22::ContractSupportedInterface {
            supported_interface: cw83::INTERFACE_NAME.into(),
            version: "0.0.0".into(),
        }],
    )
    .unwrap();

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            owner: ACCOUNT.into(),
            account_data: Binary::from_base64(PUBKEY).unwrap(),
            token_info: TokenInfo {
                collection: "test".into(),
                id: "test".into(),
            },
        },
    )
    .unwrap();

    let msg = QueryMsg::ValidSignature {
        data: to_json_binary(&MSG).unwrap(),
        signature: Binary::from_base64(SIGNATURE).unwrap(),
        payload: None,
    };

    let query_res = query(deps.as_ref(), env.clone(), msg).unwrap();

    let res: ValidSignatureResponse = from_json(query_res).unwrap();

    assert!(res.is_valid)
}
