import { assert, beforeAll, describe, expect, test } from 'vitest';
import { getWallet, getClient } from '../src/index';
import { SigningArchwayClient } from "@archwayhq/arch3.js";
import { Secp256k1HdWallet } from "@cosmjs/amino";
import { createAccountMsg, getArb36Credential, getCWFee } from '../src/msgs';
import type { MsgExecuteContractEncodeObject } from '@cosmjs/cosmwasm-stargate/build/modules';
import { MsgExecuteContract } from "cosmjs-types/cosmwasm/wasm/v1/tx";
import { toBinary } from '@cosmjs/cosmwasm-stargate';
import { GasPrice, calculateFee, coins } from '@cosmjs/stargate';
import { findAttribute } from '@cosmjs/stargate/build/logs';
import { toAscii, toBase64, toUtf8 } from '@cosmjs/encoding';
import { escapeCharacters, sortedJsonStringify } from "@cosmjs/amino/build/signdoc";


const chain_id = process.env.PUBLIC_CHAIN_ID!;

describe('Grant tests', () => {
    let client : SigningArchwayClient | undefined;
    let wallet : Secp256k1HdWallet | undefined;
    let signer : string;

    let account : string  = "archway189h8t4cz8dyhve3hsat8am2xegfupcj6f3xmsgv2zgcl6enq77xqdpg3l9";

    beforeAll(async () => {
        wallet = await getWallet();
        client = await getClient(wallet!);
        signer = (await wallet!.getAccounts())[0].address;
    })

    test("Client Defined", async () => {
        expect(wallet).toBeDefined();
        expect(client).toBeDefined();
    })


    /* test("Can create account", async () => {

        const value : MsgExecuteContract = {
            sender: signer,
            contract: process.env.PUBLIC_REGISTRY!,
            msg: new Uint8Array(Buffer.from(
                JSON.stringify(await createAccountMsg(wallet!, signer!, "test"))
            )),
            funds: []
        }

        const msg : MsgExecuteContractEncodeObject = {
            typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
            value
        }

        const gasUsed = await client!.simulate(signer!, [msg], undefined);
        const estimatedGas = Math.round(gasUsed * 1.3);


        const res = await client!.signAndBroadcast(
            signer!,
            [msg],
            calculateFee(estimatedGas, GasPrice.fromString("900000000000aarch")),
            "",
        )
        console.log("Res:", res);
        const e = res.events.find((e) => e.type === "instantiate");
        assert(e, "No reply event found"); 

        account = e.attributes[0].value;
        expect(account).toBeDefined();
    }) */


    test("Can grant permission", async () => {

        const amount = coins("1000000000", "aarch");
        const inner_msgs = { bank: { send: { amount, to_address: signer  } }};
        
        const data = { 
            nonce: Math.round(Date.now() / 1_000_000_000).toString(),
            messages: [inner_msgs],
            chain_id, 
        };

        console.log("To Sign:", data);
        const formatted = toBinary(data)

        const { signature } = (await getArb36Credential(
            wallet!,
            signer,
            formatted,
        )).cosmos_arbitrary



        const custom_msg = {
            data: data,
            signature,
        };

        let execute = { 
            execute: { msgs: [{ custom: custom_msg }]}
        }


        const value : MsgExecuteContract = {
            sender: signer,
            contract: account!,
            msg: new Uint8Array(Buffer.from(
                JSON.stringify(execute)
            )),
            funds: amount 
        }

        const msg : MsgExecuteContractEncodeObject = {
            typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
            value
        }

        const gasUsed = await client!.simulate(signer!, [msg], undefined);
        const estimatedGas = Math.round(gasUsed * 1.3);

        const res = await client!.signAndBroadcast(
            signer!,
            [msg],
            calculateFee(estimatedGas, GasPrice.fromString("900000000000aarch")),
            "",
        )
        console.log("Res:", res);
        expect(res).toBeDefined();
    })

    


});
