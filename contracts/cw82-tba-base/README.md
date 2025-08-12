# CW82: Token Bound Account

The **Account Contract** is your personal smart wallet linked to an NFT. Think of it as a digital safe that:

- Only opens for the person who owns the linked NFT
- Holds your tokens, NFTs, and other digital assets
- Can perform transactions and interact with other protocols
- Automatically protects itself during NFT transfers

## 🌟 Key Features

### 💰 **Digital Wallet Functionality**

- **Asset Storage**: Hold tokens, NFTs, staking rewards, and more
- **Transaction Execution**: Send payments, interact with DeFi, vote in governance
- **Multi-Asset Support**: Works with any CosmWasm-compatible assets
- **Fee Grants**: Can pay transaction fees for other accounts

### 🔐 **Smart Security System**

- **NFT-Based Access**: Only the NFT owner can control the account
- **Auto-Freeze Protection**: Freezably by anyone if NFT moves to escrow/marketplace
- **Owner Recovery**: Real NFT owner can always unfreeze their account
- **Transaction Validation**: Blocks potentially dangerous operations

## 🛡️ How Security Works

### **Automatic Protection During Sales** 🛡️

When you list your NFT for sale:

1. **NFT moves to marketplace escrow** 📦
2. **Account detects owner change** 🚨
3. **Anyone can freeze the account** ❄️ (including the marketplace)
4. **Account becomes read-only** 🔒
5. **New owner can unfreeze after purchase** ✅

**Why this is good:**

- Prevents you from emptying the account during a sale
- Protects buyers from purchasing "empty" NFTs
- Ensures honest trading on marketplaces

## 📱 What You Can Do

### **Basic Operations**

- **Check Status**: See if your account is active or frozen
- **View Assets**: List all tokens and NFTs in your account
- **Check Ownership**: Verify you control the account
- **View History**: See your transaction and interaction history

### **Asset Management**

- **Receive Tokens**: Anyone can send assets to your account
- **Send Tokens**: Transfer assets to other accounts (owner only)
- **NFT Operations**: Transfer, send to contracts, manage collections
- **Token Tracking**: Keep track of received NFTs automatically

### **Advanced Features**

- **Staking**: Delegate tokens to validators and earn rewards
- **Governance**: Vote on proposals using your account's assets
- **DeFi Integration**: Interact with lending, trading, and other protocols
- **Fee Management**: Grant fee allowances to other accounts

## 🎮 How to Use Your Account

### **Checking Your Account Status**

```bash
ACCOUNT_ADDRESS="your-tba-address"

# Tells if the account is currently frozen
starsd q wasm contract-state smart $ACCOUNT_ADDRESS '{"status": {}}'


# Information about account ownership. Should match a current or a pending owner
starsd q wasm contract-state smart $ACCOUNT_ADDRESS '{"ownership": {}}'
```

### **Viewing Your Assets**

```bash
# See all NFTs known to your account
starsd q wasm contract-state smart $ACCOUNT_ADDRESS '{
  "known_tokens": {"skip": 0, "limit": 50}
}'

# See the NFTs and bank balances at the same time
starsd q wasm contract-state smart $ACCOUNT_ADDRESS '{
  "assets": {"skip": 0, "limit": 50}
}'
```

### **Executing Transactions**

```bash
# Send tokens to someone
starsd tx wasm execute $ACCOUNT_ADDRESS '{
  "execute": {
    "msgs": [{
      "bank": {
        "send": {
          "to_address": "recipient-address",
          "amount": [{"denom": "ustars", "amount": "1000000"}]
        }
      }
    }]
  }
}' --from your-wallet

# Transfer an NFT
starsd tx wasm execute $ACCOUNT_ADDRESS '{
  "execute_native": {
    "msgs": [{
      "transfer_token": {
        "collection": "stars1...",
        "token_id": "123",
        "recipient": "recipient-address"
      }
    }]
  }
}' --from your-wallet
```

> **your-wallet** address must be the owner of the token

## 🛡️ Security Operations

### **Freeze Your Account** (Owner Only)

```bash
# Freeze an account if the stored owner doesn't match the updated NFT owner 
starsd tx wasm execute $ACCOUNT_ADDRESS '{"freeze": {}}' --from any-wallet
```

### **Unfreeze Your Account** (Owner Only)

```bash
# Unfreeze an account if you are a new owner the NFT or after abiother scenarios where that led to it being fronzen
starsd tx wasm execute $ACCOUNT_ADDRESS '{"execute_native": {"msgs": [{"unfreeze": {}}]}}' --from your-wallet
```

### **Anyone Can Freeze** (If NFT Owner Changed)

If the NFT is in escrow or sold to someone else, anyone can freeze the account:

```bash
# This will work only if the NFT owner has changed
starsd tx wasm execute $ACCOUNT_ADDRESS '{"freeze": {}}' --from any-address
```

## 🧮 Anti-Tampering System

### **Account Number Tracking**

Your account has a "version number" that increases with every action:

```bash
# Check current account number
starsd q wasm contract-state smart $ACCOUNT_ADDRESS '{"account_number": {}}'
```

**How traders use this:**

1. **Before trade**: Record account number (e.g., 42)
2. **Execute trade**: Perform marketplace transaction
3. **After trade**: Check account number again
4. **Verify integrity**: If still 42, no one interfered; if 43+, someone used the account

This prevents:

- Sellers from draining accounts during sales
- Buyers from being cheated with modified accounts
- Race conditions in automated trading

## 🔧 Advanced Features

### **Fee Grants**

```bash
# Allow another address to use your account for fees
starsd tx wasm execute $ACCOUNT_ADDRESS '{
  "execute_native": {
    "msgs": [{
      "fee_grant": {
        "grantee": "other-address",
        "allowance": {
          "spend_limit": [{"denom": "ustars", "amount": "1000000"}],
          "expiration": {"at_time": "1234567890"}
        }
      }
    }]
  }
}' --from your-wallet
```
