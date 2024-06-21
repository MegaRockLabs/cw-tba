import { beforeAll, describe, expect, test } from 'vitest';
import { getWallet, getClient } from '../src/index';
import { SigningArchwayClient } from "@archwayhq/arch3.js";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";




describe('Grant tests', () => {
    let client : SigningArchwayClient | undefined;
    let wallet : DirectSecp256k1HdWallet | undefined;

    beforeAll(async () => {
        wallet = await getWallet();
        client = await getClient(wallet!);
    })

    test("Clinets Defines", async () => {
        expect(wallet).toBeDefined();
        expect(client).toBeDefined();
    })



});
