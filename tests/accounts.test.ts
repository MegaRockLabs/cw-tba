import { beforeAll, describe, expect, test } from 'vitest';
import { getChainData } from '../src/chain';
import { createAccount, mintToken } from '../src/contract';
import type { ChainData, CredentialData } from '../src/types';
import { getEthSigner } from '../src/utils';

describe('Setup Tests', () => {
	let data: ChainData;
	let collection: string;

	const signer = getEthSigner();

	beforeAll(async () => {
		data = await getChainData();
		collection = data.contracts.cw721_base.address!;
	});

	test('Eth typed creation', async () => {
		console.log('Running full setup...');
		await mintToken(data, collection, '2');

		const eth_typed_data = {
			'types': {
				'EIP712Domain': [
					{ 'name': 'name', 'type': 'string' },
					{ 'name': 'version', 'type': 'string' },
					{ 'name': 'chainId', 'type': 'uint256' },
					{ 'name': 'verifyingContract', 'type': 'address' },
				],
				'CreatePrompt': [{ 'name': 'message', 'type': 'string' }],
			},
			'primaryType': 'CreatePrompt',
			'domain': {
				'chainId': '0',
				'name': 'Token-Bound Accounts',
				'verifyingContract': '0xd7296ff158d2de4a0e192c8a15b2772600980c23',
				'version': '1.1',
			},
			'message': { 'message': 'Create TBA account' },
			'signature':
				'0+mgaZb9ZYE8jObiMqJw/B170ojkEDTcZuqzvnt+4eB2KPk7Bpr5+25yfRHhWCpDqTYpgpW95ycAtZmEuxeALxs=',
			signer,
		};

		// @ts-ignore
		const acc_data: CredentialData = { credentials: [{ eth_typed_data }], use_native: true };

		await createAccount(data, collection, '2', acc_data, false);

		expect(data.address.length).toBeGreaterThan(0);
	}, { timeout: 10000 });
});
