# Token Bound Accounts: CosmWasm

Token Bound Accounts (TBAs) revolutionizes how users interact with their digital assets by enabling the creation of smart contract-based accounts linked to any non-fungible token (NFT). This innovative toolset provides unparalleled flexibility and functionality, ensuring backward compatibility with all existing NFT collections.

Our newly developed account contracts introduce customizable credentials, allowing users to integrate popular wallets like Metamask directly into their NFT accounts. This feature simplifies the process of authorizing future actions, enhancing the user experience and security.

## 🚀 Key Features

- **NFT Token Bound Account Creation**: Seamlessly create smart contract-based accounts linked to any NFT, turning your NFTs into versatile digital identities.
- **Multi-Chain Support**: Our solution is compatible across multiple blockchain networks, providing users with a broad range of options for their digital assets.
- **Flexible Access Options**: Access your NFT account using native wallets, any Cosmos wallet via public key, or even a Metamask wallet with Metamask signing.
- **Comprehensive Asset Management**: Easily manage your digital assets by depositing, withdrawing, and transferring tokens and NFTs within your account.
- **Staking and Governance**: Stake deposited currencies with validators to earn rewards and participate in governance proposals using your NFT account, ensuring your voice is heard in the community.
- **On-Chain Identity**: Transform your NFT into a comprehensive on-chain identity capable of performing a wide range of on-chain actions.
- **Package delivery**: When you sell your NFT, you will also transfer ownership of its entire inventory to the new owner.

## 🛡️ Security & Trust

### Why Token Bound Accounts Are Secure

**Think of TBAs like a secure digital safe attached to your NFT.** Here's how we keep your assets protected:

#### 1. **Ownership Verification** 🔐

- **Simple Explanation**: Only the person who actually owns the NFT can control its account
- **How it Works**: Every action checks "Do you still own this NFT?" before allowing anything to happen
- **Protection**: Prevents unauthorized access even if someone knows your account address

#### 2. **Smart Freezing System** ❄️

- **Simple Explanation**: If your NFT gets moved to escrow (like during a sale), the account can be temporarily frozen
- **How it Works**: Anyone can freeze an account if the NFT owner has changed, but only the real owner can unfreeze it
- **Protection**: Prevents theft during NFT transfers and marketplace transactions

#### 3. **Management System** 👥

- **Simple Explanation**: Special trusted services (like marketplaces) can help manage accounts, but they still have to prove you own the NFT
- **How it Works**: Managers can create or update accounts for you, but they cannot bypass ownership checks
- **Protection**: Convenience without compromising security
- **Examples**: Some collections might want to become managers to automatically create an account for each newly minted token. In some cases a trusted entity like a marketplace can inject data related to a new's owner directly immediately after a trade

### Multi-Layer Security Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    🏛️ Registry Contract                      │
│  • Verifies NFT ownership before any account creation       │
│  • Manages trusted service providers                        │
│  • Handles account migrations and updates                   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────--------------┐
│                   🏦 Individual TBA Account                               │
│  • Stores your assets safely                                              │
│  • Executes transactions only for verified owners                         │
│  • Freezing when a NFT moves to a new owner until he gets the sole access │
│  • Maintains transaction history and version tracking                     │
└─────────────────────────────────────────────────────────────--------------┘
```

### What This Means for You

✅ **Your NFT = Your Control**: Only you can manage your TBA as long as you own the NFT
✅ **Safe Trading**: Accounts protect themselves during NFT sales
✅ **Trusted Services**: Marketplaces can help without accessing your assets
✅ **No Surprises**: Transactions either work completely or fail safely
✅ **Full Transparency**: All actions are recorded on the blockchain

## 🏗️ Architecture

This repository contains three main components:

#### [(Token-Bound) Account Registry](./contracts/cw83-tba-registry/)

##### `cw83-tba-registry`

The central hub that creates and manages all Token Bound Accounts. Think of it as the "factory" that makes your NFT accounts and keeps track of which NFTs are linked to which accounts.

#### [Basic (Token-Bound) Account](./contracts/cw82-tba-base/)

##### `cw82-tba-base`

The individual smart accounts that belong to your NFTs. These are like personal digital wallets that can hold tokens, NFTs, and execute transactions on behalf of your NFT.

#### [Credentials-based (Token-Bound) Account)](./contracts/cw82-tba-credentials/)

##### `cw82-tba-credentials`

An advanced version of the account contract with enhanced authentication features, supporting multiple wallet types and signing methods. The contract makes sure that at least one of the credential corresponds to the owner address in the NFT collection contract. The actions on the account can be performed by providing a proof of ownership to any of the registered credentials.

## 🚀 Getting Started

1. **Choose Your NFT**: Any CW721-compatible NFT can have a Token Bound Account
2. **Create Account**: Use the registry to create an account linked to your NFT
3. **Add Assets**: Deposit tokens, NFTs, or other digital assets into your account
4. **Take Actions**: Use your TBA to participate in DeFi, governance, trading, and more
5. **Transfer Together**: When you sell your NFT, the new owner gets everything in the account

---

**Token Bound Accounts bring a new level of autonomy and customization to your digital presence.** Embrace the future of on-chain identity and asset management with TBAs - where your NFTs become powerful, secure digital identities.

## 📚 Documentation

- [Registry Contract](./contracts/cw83-tba-registry/README.md)
- [Base Account](./contracts/cw82-tba-base/README.md)
- [Credential Account](./contracts/cw82-tba-credentials/README.md)
