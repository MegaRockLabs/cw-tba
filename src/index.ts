import { SigningArchwayClient } from "@archwayhq/arch3.js";
import { Secp256k1HdWallet } from "@cosmjs/amino";


let client : SigningArchwayClient | undefined;
let wallet : Secp256k1HdWallet | undefined;



export const getWallet = async () => {
    if (!wallet) {
        const mnemonic = process.env.PUBLIC_MNEMONIC;
        wallet = await Secp256k1HdWallet.fromMnemonic(mnemonic!, { prefix: "archway" });
    }
    return wallet;
}
  


export const getClient = async (wallet: Secp256k1HdWallet) => {
    if (!client) {
        const accounts = await wallet.getAccounts();
        //const granterAddress = accounts[0].address;

        const endpoint = process.env.PUBLIC_ENDPOINT!;
        client = await SigningArchwayClient.connectWithSigner(endpoint, wallet);
    }
    return client
}