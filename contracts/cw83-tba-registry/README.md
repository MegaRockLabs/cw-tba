# CW83: Token Bound Account Registry

Global on-chain registry of token bound accounts (TBAs) that keeps track of all associated accounts and their owners. Works with any CW721-compatible colection


## Usage


To create a new account you must call `ExecuteMsg::CreateAccount` with the following parameters:
- `chain_id`: The chain id of the netowrk where the registry is deployed
- `code_id`:  The code id corresponding to the account contract to be instantiated
- `msg`:  A valid `TokenAccount` object describing below


The object `TokenAccount` has the the following properties:
- `token_info`: data object with `collection` standing for the contract address and `id` standing for the token id
- `account_data`: a `CredentialData` object that contains the data used for the account creation
- `actions` (optional): a list of actions to be executed on the account creation. The actions are defined in the `ExecuteAccountMsg` enum

Information about who controls the account is stored `CredentialData` that looks like this:
- `credentials`: a list of `Credential` objects defined in `smart-account-auth`. For the most part are a `data`, `signature` and `pubkey` combos
- `primary_index`: (optional): the index of the credential that will be used as main by default
- `with_native`: (optional): a flag allowing the cuurent tx signer to call the contract directly and natively without additional signatures

Ether one of the credentials or transaction signer with `with_native` feature set must be tranformable into an address that is holding the given token. If that requirement is satisfied you can add any number of additonal credentials.

#### Example Josn
```json
{
    "chain_id": "stargaze-4",
    "code_id": 1234,
    "token_info": {
        "collection": "stars1...",
        "id": "1234"
    },

    "account_data": {
        "with_native": true
    },
}
```

With CLI tool
```bash
REGISTRY_ADDRESS=stars1...

starsd tx wasm execute $REGISTRY_ADDRESS '{ "create_account": { "chain_id" ...  }  }'  --gas-prices 0.01ustars --from test --amount 3000000000ustars --gas 35000000  -y
```


## The registry might charge fees to compensate for the reduced fees of the network, to handle requirements imposed by governance, to issue feegrants or to cover other possible expenses

You can query the set amount and other parameters by calling `QueryMsg::RegistryParmas` :
```bash
starsd q wasm contract-state smart $REGISTRY_ADDRESS '{ "registry_params": {   }  }'
```


Once the account has been created you can check an associated account by calling `QueryMsg::AccountInfo` like this:
```bash
starsd q wasm contract-state smart $REGISTRY_ADDRESS '{ "account_info": { "collection": "starg1...", "id": "1234" }  }'
```