# CW83: Token Bound Account Registry

The **Registry Contract** is the central hub that creates and manages all Token Bound Accounts. Think of it as a secure factory that:

- Creates new accounts linked to your NFTs
- Keeps track of which account belongs to which NFT
- Ensures only legitimate NFT owners can control accounts
- Manages trusted service providers (like marketplaces)

## 🌟 Key Features

### 🏭 **Account Factory**

- Creates smart contract accounts linked to any CW721-compatible NFT
- Each NFT can have exactly one account (but accounts can be reset/upgraded)
- Generates unique addresses for each NFT account

### 🔐 **Security Guardian**

- **Ownership Verification**: Always checks that you actually own the NFT before creating/updating accounts
- **Manager System**: Trusted services (marketplaces, minting platforms) can help users but cannot bypass ownership checks
- **Fee Protection**: Validates proper fees are paid and prevents economic attacks
- **Code ID Validation**: Only allows accounts to be created with approved, secure contract code

### 🛡️ **Trust & Safety Features**

#### **NFT Ownership Verification**

```
Before ANY action:
1. "Do you own this NFT?" ✅
2. "Is the NFT in your wallet?" ✅  
3. "Are you the real owner?" ✅
Only then: "OK, you can proceed"
```

#### **Trusted Manager System**

- **Who**: Verified marketplaces, minting platforms, or other trusted services
- **What they can do**: Create accounts for users, help with ownership transfers
- **What they CANNOT do**: Bypass NFT ownership checks or take control without permission
- **Your protection**: Managers must prove you own the NFT for every action

#### **Reset & Migration Protection**

- **Reset**: Safely replace an old account with a new one (old data is permanently deleted)
- **Migration**: Upgrade account to newer versions with improved features
- **Safety**: Only the NFT owner can trigger these actions

## 📋 How to Use

### Creating Your First Account

**Simple Method (CLI):**

```bash
REGISTRY_ADDRESS=stars1...  # The registry contract address

starsd tx wasm execute $REGISTRY_ADDRESS '{
  "create_account": {
    "chain_id": "stargaze-1",
    "code_id": 1234,
    "account_data": {
      "token_info": {
        "collection": "stars1...",  # Your NFT contract address
        "id": "1234"               # Your NFT token ID
      },
      "credential_data": {
        "credentials": [
          {
            "data": "...", // Your wallet signature
            "signature": "...", // Proof you control the wallet
            "pubkey": "..." // Your public key
            ...  // other fields depending on the authentication method
          }
        ],
        "with_native": true         # Use current wallet as a credential that can do actions directly without additional signatures
      }
    }
  }
}'  --from your-wallet --amount 3000000ustars  # In case registry charge fees. Otherwise forwarded to your new account
```

### What This Does:

1. **Verifies** you own NFT #1234 from collection "stars1..."
2. **Creates** a smart contract account linked to that NFT
3. **Sets up** authentication so you can control it with your owner wallet (or alternative credentials)
4. **Charges** pays creation fees if exist and forwards any excess funds to your new account

### Advanced Options

#### **Multiple Authentication Methods**

```json
{
  "account_data": {
    "credentials": [
      {
        "data": "...", // Your wallet signature
        "signature": "...", // Proof you control the wallet
        "pubkey": "..." // Your public key
      }
    ],
    "with_native": true // Also allow direct wallet access
  }
}
```

### Checking Your Account

**Find your account address:**

```bash
starsd q wasm contract-state smart $REGISTRY_ADDRESS '{
  "account_info": {
    "collection": "stars1...",  # Your NFT contract
    "id": "1234"               # Your NFT ID
  }
}'
```

**Check registry settings:**

```bash
starsd q wasm contract-state smart $REGISTRY_ADDRESS '{
  "registry_params": {}
}'
```

## 💰 Fees & Costs

The registry charges small fees to:

- Cover network gas costs for account creation
- Support ongoing development and security
- Provide fee grants for your account operations

**Fee Structure:**

- Creation fees are set by governance
- Fees can be paid in any token and user can choose any of the set options
- Any excess funds are forwarded to a newly created account
- You can query current configuration using `registry_params`

## 🔄 Account Lifecycle

### 1. **Creation**

- Verify NFT ownership ✅
- Pay creation fee 💰
- Deploy smart contract account 🚀

### 2. **Usage**

- Deposit assets into your account 💎
- Execute transactions through your account ⚡
- Participate in DeFi, governance, trading 📈

### 3. **Ownership Transfer**

- When you sell your NFT 💸
- Account ownership automatically transfers 🔄
- New owner gets full control + all assets 📦

### 4. **Upgrade / Migrations**

- The owner can use the migration engine to update an account to a newer version 🆙
- Migration is only possible to an authorised code_id that is set by governance 🔄
- Each account must be migrated individually

## 🛡️ Security Guarantees

### ✅ **What's Protected:**

- Only NFT owners can control accounts
- Managers cannot bypass ownership checks
- All operations are atomic (complete or fail entirely)
- Fee validation prevents economic attacks
- Code validation ensures only secure account types

### ❌ **What's NOT Protected:**

- NFT marketplaces or collections themselves (that's outside our scope)
- Private keys or wallet security (that's your responsibility)
- Smart contract bugs in other protocols you interact with

### 🚨 **Emergency Features:**

- **Freeze Protection**: Accounts can be frozen if NFT ownership changes unexpectedly
- **Admin Functions**: Trusted governance can update allowed code IDs and managers (optional)
- **Migration Support**: Accounts can be upgraded to fix issues or add features

## 🧑‍💻 For Developers

### **Integration Examples:**

**Marketplace Integration:**

```rust
// Check if NFT has an account
let account_info = registry.account_info(collection, token_id)?;

// Create account for user during NFT mint
let create_msg = ExecuteMsg::CreateAccount {
    chain_id: "your-chain".to_string(),
    code_id: approved_code_id,
    account_data: TokenAccountPayload {
        token_info: TokenInfo { collection, id },
        credential_data: user_credentials,
        create_for: Some(user_address),
        actions: Some(vec![...]), // Optional initial actions
    }
};
```

**Ownership Transfer:**

```rust
// Update account when NFT is sold
let transfer_msg = ExecuteMsg::UpdateAccountOwnership {
    token_info: TokenInfo { collection, id },
    new_account_data: Some(new_owner_credentials),
    update_for: Some(new_owner_address),
};
```

---

The Registry Contract ensures that your Token Bound Accounts are created safely, managed securely, and transferred properly. It's the trusted foundation that makes the entire TBA ecosystem possible.
