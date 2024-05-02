# Stargaze Token Bound Accounts

Stargaze contracts implementing [cw82](https://github.com/MegaRockLabs/cw-extra/tree/main/packages/cw82) and [cw83](https://github.com/MegaRockLabs/cw-extra/tree/main/packages/cw83) 


## Token Registry

Token-Bound Account Registry that follows [cw83](https://github.com/MegaRockLabs/cw-extra/tree/main/packages/cw83) account registry standard.  

### **Features:**

  - Creating a smart contract based account for any NFT
  - Backwards compatability and support for existing collections
  - Ability to query all accounts, filter by a collection and get all known collections
  - Updating the smart account owner if NFT owner changes
  - Posssiblity to migrate accounts to another implementation
  - Ability to reset an account linked to an NFT (after purging the state of existing one)

## Token Account

A smart contract based account following [cw82](https://github.com/MegaRockLabs/cw-extra/tree/main/packages/cw82). 

Uses a secp256k1 public key and support arbitrary signature verification defined in [036](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-036-arbitrary-signature.md) 

As a minimal proxy following [cw1](https://github.com/CosmWasm/cw-plus/tree/main/packages/cw1) allows executing messages with the smart account as an initiator (msg.sender) but only for the owner of an NFT the account is linked to. Ownership verification happens through the registry and that must be notified for the new owneship to take place.

The contract implementation allows accounts to become aware of other tokens in their posession. This allows users to directly query the (recognised) asset inventory and gives a flexibilty for the owner to explicitely associate itself with certain tokens

### **Features:**

  - Allowing the NFT owner to initiate any Bank, Staking or IBC messages
  - Functions for being frozen if the current contract owner is different from the NFT owner
  - Using provided key for signature verification to allow sign-ins to apps that support `cw81`
  - Concept of asset awareness to instantly query the inventory of an account
  - NFT transfer and sent functionalities for the tokens held in account inventory
  - Sending mint messages firectly from the account (and instant token awareness on success)

