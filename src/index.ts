import type { AminoMsg, OfflineAminoSigner, StdFee, StdSignDoc } from '@cosmjs/amino';
import { toBase64, toUtf8 } from '@cosmjs/encoding';
import type { CosmosArbitrary, Credential, MsgSignData } from './types';

const base64regex = /^([0-9a-zA-Z+/]{4})*(([0-9a-zA-Z+/]{2}==)|([0-9a-zA-Z+/]{3}=))?$/;

export const makeSignDoc = (
	msgs: readonly AminoMsg[],
	fee: StdFee,
	chainId: string,
	memo: string | undefined,
	accountNumber: number | string,
	sequence: number | string,
	timeout_height?: bigint,
): StdSignDoc => {
	return {
		chain_id: chainId,
		account_number: accountNumber.toString(),
		sequence: sequence.toString(),
		fee: fee,
		msgs: msgs,
		memo: memo || '',
		...(timeout_height && { timeout_height: timeout_height.toString() }),
	};
};

export const getArb36SignData = (signer: string, data: string | Uint8Array): MsgSignData => ({
	type: 'sign/MsgSignData',
	value: {
		signer,
		data: typeof data === 'string'
			? (base64regex.test(data) ? data : toBase64(toUtf8(data)))
			: toBase64(data),
	},
});

export const getArb36SignDoc = (signerAddress: string, data: string | Uint8Array): StdSignDoc => {
	const msg = getArb36SignData(signerAddress, data);
	return makeSignDoc([msg], { gas: '0', amount: [] }, '', '', 0, 0);
};

export const getCosmosArbitrary = async (
	signer: OfflineAminoSigner,
	chainId: string,
	message: string | Uint8Array,
	signerAddress?: string,
	hrp?: string,
): Promise<Credential & { cosmos_arbitrary: CosmosArbitrary }> => {
	const accounts = await (signer as OfflineAminoSigner).getAccounts();
	const firstAccount = accounts[0];
	signerAddress ??= firstAccount.address;

	const pubkey = toBase64(firstAccount.pubkey);

	hrp ??= signerAddress.split('1')[0];

	const signResult = await (signer as OfflineAminoSigner) // OfflineAminoSigner + AminoWallet
		.signAmino(signerAddress, getArb36SignDoc(signerAddress, message));

	const signature = signResult.signature.signature;

	const cosmos_arbitrary: CosmosArbitrary = {
		message: typeof message === 'string'
			? (base64regex.test(message) ? message : toBase64(toUtf8(message)))
			: toBase64(message),
		pubkey,
		signature,
		address: signerAddress,
	};

	return { cosmos_arbitrary };
};
