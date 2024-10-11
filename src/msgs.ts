import type { SigningArchwayClient, StdFee } from "@archwayhq/arch3.js";
import type { CreateAccountMsg, Credential, CredentialData, MsgSignData } from "./types";
import type { Secp256k1HdWallet, StdSignDoc, StdTx } from "@cosmjs/amino";
import { makeSignDoc, serializeSignDoc, makeStdTx } from "@cosmjs/amino";
import { escapeCharacters, sortedJsonStringify } from "@cosmjs/amino/build/signdoc";

import { Secp256k1, Secp256k1Signature, sha256 } from "@cosmjs/crypto"
import { fromBase64, toAscii, toBase64 } from "@cosmjs/encoding";
import { toBinary } from "@cosmjs/cosmwasm-stargate";

const accountNumber = 0;
const sequence = 0;
const chainId = "";
const fee: StdFee = { gas: "0", amount: [] };
const memo = "";



export const createAccountMsg = async (
  signer:   Secp256k1HdWallet,
  signerAddress: string,
  message: string,
) => {

  const creds : CredentialData = {
    credentials: [await getArb36Credential(
      signer, 
      signerAddress, 
      message
    )],
  }


  const execute_msg : CreateAccountMsg<string>  = {
      create_account: {
          chain_id: process.env.PUBLIC_CHAIN_ID!,
          code_id: Number(process.env.PUBLIC_ACCOUNT_ID!),
          msg: {
            account_data: toBinary(creds),
            token_info: {
              collection: process.env.PUBLIC_NFT_COLLECTION!,
              id: process.env.PUBLIC_NFT_ID!,
            },
          }
      }
  }
  

  return execute_msg;
}


export const getCWFee = async (
    client: SigningArchwayClient, 
    signer: string
) : Promise<any> => {
    return "auto"
}


export const getArb36SignData = (
    signerAddress: string,
    data: string | Uint8Array,
  ) : MsgSignData => ({
    type: "sign/MsgSignData",
    value: {
      signer: signerAddress,
      data: typeof data === "string" ? data : toBase64(data),
    }
  })


export const getArb36SignDoc = (
    signerAddress: string,
    data: Uint8Array,
  ) : StdSignDoc => {
    const msg = getArb36SignData(signerAddress, data);
    return makeSignDoc([msg], fee, chainId, memo, accountNumber, sequence);
  }


export const getArb36Credential = async (
    signer:   Secp256k1HdWallet,
    signerAddress: string,
    message: string | Uint8Array,
  ) : Promise<Credential> => {
    
    const data = typeof message === "string" ? toAscii(message) : message;
    const signDoc = getArb36SignDoc(signerAddress, data);
    const signRes = await signer.signAmino(signerAddress, signDoc);


    const stdTx = makeStdTx(signDoc, signRes.signature);
    
    const ok = await verifyAdr36Tx(stdTx);
    if (!ok) {
      throw new Error("Signature verification failed locally");
    }
    
    const res = {
      signature: signRes.signature.signature,
      pubkey: signRes.signature.pub_key.value,
      message: toBase64(data),
      hrp: "archway"
    }

    return { "cosmos_arbitrary" : res  }
  }


  
  export const verifyAdr36Tx = async (signed: StdTx): Promise<boolean> => {
    // Restrictions from ADR-036
    if (signed.memo !== "") throw new Error("Memo must be empty.");
    if (signed.fee.gas !== "0") throw new Error("Fee gas must 0.");
    if (signed.fee.amount.length !== 0) throw new Error("Fee amount must be an empty array.");
  
    const accountNumber = 0;
    const sequence = 0;
    const chainId = "";
  
    // Check `msg` array
    const signedMessages = signed.msg;
    if (signedMessages.length === 0) {
      throw new Error("No message found. Without messages we cannot determine the signer address.");
    }
  
    const signatures = signed.signatures;
    if (signatures.length !== 1) throw new Error("Must have exactly one signature to be supported.");
    const signature = signatures[0];
   
  
    const signBytes = serializeSignDoc(
      makeSignDoc(signed.msg, signed.fee, chainId, signed.memo, accountNumber, sequence),
    );
    const prehashed = sha256(signBytes);
  
    const secpSignature = Secp256k1Signature.fromFixedLength(fromBase64(signature.signature));
    const rawSecp256k1Pubkey = fromBase64(signature.pub_key.value);
  
    const ok = await Secp256k1.verifySignature(secpSignature, prehashed, rawSecp256k1Pubkey);
    return ok;
  }