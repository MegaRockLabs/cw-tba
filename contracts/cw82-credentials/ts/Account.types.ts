
export type Credential = {
  caller: Caller;
} | {
  evm: EvmCredential;
} | {
  secp256k1: Secp256K1;
} | {
  ed25519: Ed25519;
} | {
  cosmos_arbitrary: CosmosArbitrary;
};


export interface CredentialData {
  credentials: Credential[];
  primary_index?: number | null;
  secs_to_expire?: number | null;
  with_caller?: boolean | null;
}

export interface Caller {
  id: number[];
}
export interface EvmCredential {
  message: number[];
  signature: number[];
  signer: number[];
}
export interface Secp256K1 {
  hrp?: string | null;
  message: number[];
  pubkey: number[];
  signature: number[];
}
export interface Ed25519 {
  message: number[];
  pubkey: number[];
  signature: number[];
}
export interface CosmosArbitrary {
  hrp?: string | null;
  message: number[];
  pubkey: number[];
  signature: number[];
}
export interface TokenInfo {
  collection: string;
  id: string;
}


export type ExecuteMsg = {
  execute: {
    msgs: SignedCosmosMsgs[];
  };
} | {
  mint_token: {
    minter: string;
    msg: Binary;
  };
} |  {
  transfer_token: {
    collection: string;
    recipient: string;
    token_id: string;
  };
} | {
  forget_tokens: {
    collection: string;
    token_ids: string[];
  };
} | {
  update_known_tokens: {
    collection: string;
    limit?: number | null;
    start_after?: string | null;
  };
} | {
  update_ownership: {
    new_account_data?: CredentialData | null;
    new_owner: string;
  };
} | {
  update_account_data: {
    new_account_data: CredentialData;
  };
} | {
  receive_nft: Cw721ReceiveMsg;
} | {
  freeze: {};
} | {
  unfreeze: {};
} | {
  purge: {};
} | {
  extension: {
    msg: SignedAccountActions;
  };
};

export type SignedCosmosMsgs = {
  bank: BankMsg;
} | {
  custom: SignedCosmosMsgs;
} | {
  staking: StakingMsg;
} | {
  distribution: DistributionMsg;
} | {
  stargate: {
    type_url: string;
    value: Binary;
    [k: string]: unknown;
  };
} | {
  ibc: IbcMsg;
} | {
  wasm: WasmMsg;
} | {
  gov: GovMsg;
};
export type BankMsg = {
  send: {
    amount: Coin[];
    to_address: string;
    [k: string]: unknown;
  };
} | {
  burn: {
    amount: Coin[];
    [k: string]: unknown;
  };
};
export type Uint128 = number;
export type CosmosMsgForEmpty = {
  bank: BankMsg;
} | {
  custom: Empty;
} | {
  staking: StakingMsg;
} | {
  distribution: DistributionMsg;
} | {
  stargate: {
    type_url: string;
    value: Binary;
    [k: string]: unknown;
  };
} | {
  ibc: IbcMsg;
} | {
  wasm: WasmMsg;
} | {
  gov: GovMsg;
};


export type StakingMsg = {
  delegate: {
    amount: Coin;
    validator: string;
    [k: string]: unknown;
  };
} | {
  undelegate: {
    amount: Coin;
    validator: string;
    [k: string]: unknown;
  };
} | {
  redelegate: {
    amount: Coin;
    dst_validator: string;
    src_validator: string;
    [k: string]: unknown;
  };
};
export type DistributionMsg = {
  set_withdraw_address: {
    address: string;
    [k: string]: unknown;
  };
} | {
  withdraw_delegator_reward: {
    validator: string;
    [k: string]: unknown;
  };
};
export type Binary = string;
export type IbcMsg = {
  transfer: {
    amount: Coin;
    channel_id: string;
    timeout: IbcTimeout;
    to_address: string;
    [k: string]: unknown;
  };
} | {
  send_packet: {
    channel_id: string;
    data: Binary;
    timeout: IbcTimeout;
    [k: string]: unknown;
  };
} | {
  close_channel: {
    channel_id: string;
    [k: string]: unknown;
  };
};
export type Timestamp = Uint64;
export type Uint64 = string;
export type WasmMsg = {
  execute: {
    contract_addr: string;
    funds: Coin[];
    msg: Binary;
    [k: string]: unknown;
  };
} | {
  instantiate: {
    admin?: string | null;
    code_id: number;
    funds: Coin[];
    label: string;
    msg: Binary;
    [k: string]: unknown;
  };
} | {
  migrate: {
    contract_addr: string;
    msg: Binary;
    new_code_id: number;
    [k: string]: unknown;
  };
} | {
  update_admin: {
    admin: string;
    contract_addr: string;
    [k: string]: unknown;
  };
} | {
  clear_admin: {
    contract_addr: string;
    [k: string]: unknown;
  };
};
export type GovMsg = {
  vote: {
    proposal_id: number;
    vote: VoteOption;
    [k: string]: unknown;
  };
};
export type VoteOption = "yes" | "no" | "abstain" | "no_with_veto";
export type ExecuteAccountMsgForEmptyAndNullable_EmptyAndBinary = {
  execute: {
    msgs: CosmosMsgForEmpty[];
  };
} | {
  mint_token: {
    minter: string;
    msg: Binary;
  };
} | {
  send_token: {
    collection: string;
    contract: string;
    msg: Binary;
    token_id: string;
  };
} | {
  transfer_token: {
    collection: string;
    recipient: string;
    token_id: string;
  };
} | {
  forget_tokens: {
    collection: string;
    token_ids: string[];
  };
} | {
  update_known_tokens: {
    collection: string;
    limit?: number | null;
    start_after?: string | null;
  };
} | {
  update_ownership: {
    new_account_data?: Binary | null;
    new_owner: string;
  };
} | {
  update_account_data: {
    new_account_data: Binary;
  };
} | {
  receive_nft: Cw721ReceiveMsg;
} | {
  freeze: {};
} | {
  unfreeze: {};
} | {
  purge: {};
} | {
  extension: {
    msg?: Empty | null;
  };
};
export interface Coin {
  amount: Uint128;
  denom: string;
  [k: string]: unknown;
}
export interface SignedCosmosMsgs {
  data: CosmosMsgDataToSign;
  payload?: AuthPayload | null;
  signature: Binary;
}
export interface CosmosMsgDataToSign {
  chain_id: string;
  messages: CosmosMsgForEmpty[];
  timestamp: Timestamp;
}
export interface Empty {
  [k: string]: unknown;
}
export interface IbcTimeout {
  block?: IbcTimeoutBlock | null;
  timestamp?: Timestamp | null;
  [k: string]: unknown;
}
export interface IbcTimeoutBlock {
  height: number;
  revision: number;
  [k: string]: unknown;
}
export interface AuthPayload {
  address?: string | null;
  credential_id?: number[] | null;
  hrp?: string | null;
}
export interface Cw721ReceiveMsg {
  msg: Binary;
  sender: string;
  token_id: string;
}
export interface SignedAccountActions {
  data: AccountActionDataToSign;
  payload?: AuthPayload | null;
  signature: Binary;
}
export interface AccountActionDataToSign {
  actions: ExecuteAccountMsgForEmptyAndNullable_EmptyAndBinary[];
  chain_id: string;
  timestamp: Timestamp;
}
export type QueryMsg = {
  status: {};
} | {
  token: {};
} | {
  registry: {};
} | {
  known_tokens: {
    limit?: number | null;
    skip?: number | null;
  };
} | {
  assets: {
    limit?: number | null;
    skip?: number | null;
  };
} | {
  full_info: {
    limit?: number | null;
    skip?: number | null;
  };
} | {
  serial: {};
} | {
  extension: {
    msg: Empty;
  };
} | {
  can_execute: {
    msg: SignedCosmosMsgs;
    sender: string;
  };
} | {
  valid_signature: {
    data: Binary;
    payload?: Binary | null;
    signature: Binary;
  };
} | {
  valid_signatures: {
    data: Binary[];
    payload?: Binary | null;
    signatures: Binary[];
  };
} | {
  ownership: {};
};
export interface MigrateMsg {
  params?: Empty | null;
}
export interface AssetsResponse {
  balances: Coin[];
  tokens: TokenInfo[];
}
export interface CanExecuteResponse {
  can_execute: boolean;
}
export type Null = null;
export type Addr = string;
export type Expiration = {
  at_height: number;
} | {
  at_time: Timestamp;
} | {
  never: {};
};
export interface FullInfoResponse {
  balances: Coin[];
  ownership: OwnershipForAddr;
  registry: string;
  status: Status;
  token_info: TokenInfo;
  tokens: TokenInfo[];
}
export interface OwnershipForAddr {
  owner?: Addr | null;
  pending_expiry?: Expiration | null;
  pending_owner?: Addr | null;
}
export interface Status {
  frozen: boolean;
}
export type ArrayOfTokenInfo = TokenInfo[];
export interface OwnershipForString {
  owner?: string | null;
  pending_expiry?: Expiration | null;
  pending_owner?: string | null;
}
export type String = string;
export interface ValidSignatureResponse {
  is_valid: boolean;
}
export interface ValidSignaturesResponse {
  are_valid: boolean[];
}