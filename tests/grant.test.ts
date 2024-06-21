import { beforeAll, describe, expect, test } from 'vitest';
import { getWallet, getClient } from '../src/index';
import { SigningArchwayClient } from "@archwayhq/arch3.js";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { createAccountMsg, getCWFee } from '../src/msgs';




describe('Grant tests', () => {
    let client : SigningArchwayClient | undefined;
    let wallet : DirectSecp256k1HdWallet | undefined;
    let signer : string | undefined;

    let account : string | undefined;

    beforeAll(async () => {
        wallet = await getWallet();
        client = await getClient(wallet!);
        signer = (await wallet!.getAccounts())[0].address;
    })

    test("Client Defined", async () => {
        expect(wallet).toBeDefined();
        expect(client).toBeDefined();
    })

    test("Can create account", async () => {

        const res = await client!.execute(
            signer!,
            process.env.PUBLIC_REGISTRY!,
            createAccountMsg(),
            await getCWFee(client!, signer!)
        )

        expect(res).toBeDefined();
    })

    


});
