import type { AminoMsg } from "@cosmjs/amino";

type Arbitrary = {
    signature: string;
    pubkey: string;
    message: string;
    hrp: string;
}

export type Credential = {
    "cosmos_arbitrary": Arbitrary
} 

export interface MsgSignData extends AminoMsg {
    readonly type: "sign/MsgSignData";
    readonly value: {
      signer: string;
      data: string;
    };
}

export type TokenInfo = {
  collection: string;
  id: string;
}


export type CredentialData = {
  credentials     : Credential[];
  primary_index?  : number | null;
  with_caller?    : boolean | null;
}


export type TokenAccount<T = string> = {
  account_data : T;
  token_info   : TokenInfo;
  create_for?  : string | null;
}

export type CreateAccount<T = string> = {
  chain_id: string;
  code_id: number;
  msg: TokenAccount<T>;
}


export type CreateAccountMsg<T = CredentialData> =  {
  "create_account": CreateAccount<T>;
}