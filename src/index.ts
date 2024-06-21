import { SigningArchwayClient } from "@archwayhq/arch3.js";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";


let client : SigningArchwayClient | undefined;
let wallet : DirectSecp256k1HdWallet | undefined;



export const getWallet = async () => {
    if (!wallet) {
        const mnemonic = process.env.PUBLIC_MNEMONIC;
        wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic!, { prefix: "archway" });
    }
    return wallet;
}
  


export const getClient = async (wallet: DirectSecp256k1HdWallet) => {
    if (!client) {
        const accounts = await wallet.getAccounts();
        const granterAddress = accounts[0].address;

        console.log("granterAddress: ", granterAddress);
        const endpoint = process.env.PUBLIC_ENDPOINT!;
        console.log("endpoint: ", endpoint);
        client = await SigningArchwayClient.connectWithSigner(endpoint, wallet);
    }
    return client
}