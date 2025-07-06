import { toBase64, toUtf8 } from '@cosmjs/encoding';
import { getCosmosArbitraryCredential } from 'smart-account-auth';
import { getCosmosArbitrary } from '.';
import {
	createAccount,
	instantiateCollection,
	instantiateRegistry,
	mintToken,
	queryAccountInfo,
	queryTokenOwner,
} from './contract';
import { type ChainData, type MsgDataToSign } from './types';

// Full test setup helper
export const initialSetup = async (chain: ChainData) => {
	if (!chain.contracts.cw83_tba_registry.address) {
		await instantiateRegistry(chain);
	}

	const collection = chain.contracts.cw721_base.address || (await instantiateCollection(chain));

	await mintToken(chain, collection, '1');

	const msgToSign: MsgDataToSign = {
		chain_id: chain.config.chain_id,
		contract_address: chain.contracts.cw83_tba_registry.address,
		messages: ['Create TBA account'],
		nonce: '0',
	};

	const { cosmos_arbitrary: c } = await getCosmosArbitrary(
		chain.wallet,
		chain.config.chain_id,
		toUtf8(JSON.stringify(msgToSign)),
		chain.address,
		chain.config.prefix,
	);

	const accs = await chain.wallet.getAccounts();

	const cred = {
		cosmos_arbitrary: {
			message: c.message,
			signature: c.signature,
			pubkey: c.pubkey,
			address: accs[0].address,
		},
	};

	// console.log('Contracts:', chain.contracts);
	await createAccount(chain, collection, '1', { credentials: [cred] });
};
