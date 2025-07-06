import { sleep } from 'bun';
import { beforeAll, describe, expect, test } from 'vitest';
import { getChainData } from '../src/chain';
import {
	createAccount,
	executeSignedAction,
	mintToken,
	queryTokenOwner,
	sendToken,
	transferToken,
} from '../src/contract';
import type { ChainData, ExecuteAccountMsg, FullInfoResponse } from '../src/types';
import { getEthSigner } from '../src/utils';

describe('Setup Tests', () => {
	const signer = getEthSigner();

	let chain: ChainData;
	let collection: string;
	let cred_acc: string;

	beforeAll(async () => {
		chain = await getChainData();
		collection = chain.contracts.cw721_base.address!;
		await sleep(4500);

		cred_acc = chain.contracts.cw82_tba_credentials.address;
		expect(cred_acc).toBeDefined();
	});

	test('Eth typed cosmos msg', async () => {
		const info: FullInfoResponse = await chain.queryClient.wasm.queryContractSmart(cred_acc, {
			full_info: {},
		});
		if (info.credentials.account_number != 1) return;

		const eth_typed_data = {
			'types': {
				EIP712Domain: [
					{ 'name': 'name', 'type': 'string' },
					{ 'name': 'version', 'type': 'string' },
					{ 'name': 'chainId', 'type': 'uint256' },
					{ 'name': 'verifyingContract', 'type': 'address' },
				],
				Coin: [{ name: 'denom', type: 'string' }, { name: 'amount', type: 'uint256' }],
				Send: [{ name: 'to_address', type: 'string' }, { name: 'amount', type: 'Coin[]' }],
				BankMsg: [{ name: 'send', type: 'Send' }],
				CosmosMsg: [{ name: 'bank', type: 'BankMsg' }],
				Execute: [{ name: 'msgs', type: 'CosmosMsg[]' }],
				AccountAction: [{ name: 'execute', type: 'Execute' }],
			},
			'primaryType': 'AccountAction',
			'domain': {
				'verifyingContract': '0xf43f49b4651e4bffc120413515f1a7ec2340747f',
				'name': 'Token-Bound Accounts',
				'version': '1.1',
				'chainId': '0',
			},
			'message': {
				'execute': {
					'msgs': [{
						'bank': {
							'send': {
								'to_address': 'stars16z43tjws3vw06ej9v7nrszu0ldsmn0eyjnjpu8',
								'amount': [{ 'denom': 'ustars', 'amount': '500000' }],
							},
						},
					}],
				},
			},
			'signature':
				'nCeN4Jb0+3sx+fx9pRxBCKh4LqdB8ky3ax0Iwk1bjXNfYyHB62roHFcKmAt5ZnZbubJzlTmh6CdBzZDoIUMkvRs=',
			signer,
		};

		const msg: ExecuteAccountMsg = { execute: eth_typed_data.message.execute };
		await executeSignedAction(chain, msg, { eth_typed_data }, [{
			denom: 'ustars',
			amount: '500000',
		}]);

		expect(chain.address.length).toBeGreaterThan(0);
	});

	/* test('Eth typed cosmos msg', async () => {
		const info: FullInfoResponse = await chain.queryClient.wasm.queryContractSmart(cred_acc, {
			full_info: {},
		});
		if (info.credentials.account_number != 1) return;

		const eth_typed_data = {
			'types': {
				EIP712Domain: [
					{ 'name': 'name', 'type': 'string' },
					{ 'name': 'version', 'type': 'string' },
					{ 'name': 'chainId', 'type': 'uint256' },
					{ 'name': 'verifyingContract', 'type': 'address' },
				],
				Coin: [{ name: 'denom', type: 'string' }, { name: 'amount', type: 'uint256' }],
				Send: [{ name: 'to_address', type: 'string' }, { name: 'amount', type: 'Coin[]' }],
				Delegate: [{ name: 'validator', type: 'string' }, { name: 'amount', type: 'Coin' }],
				BankMsg: [{ name: 'send', type: 'Send' }],
				StakingMsg: [{ name: 'delegate', type: 'Delegate' }],
				CosmosMsg: [{ name: 'bank', type: 'BankMsg' }, { name: 'staking', type: 'StakingMsg' }],
				Execute: [{ name: 'msgs', type: 'CosmosMsg[]' }],
				AccountAction: [{ name: 'execute', type: 'Execute' }],
			},
			'primaryType': 'AccountAction',
			'domain': {
				'verifyingContract': '0x73c4d31b9abcfb1d2096b69f4cf5fb2ca5d24635',
				'name': 'Token-Bound Accounts',
				'version': '1.1',
				'chainId': '0',
			},
			'message': {
				'execute': {
					'msgs': [{
						'bank': {
							'send': {
								'to_address': 'stars16z43tjws3vw06ej9v7nrszu0ldsmn0eyjnjpu8',
								'amount': [{ 'denom': 'ustars', 'amount': '500000' }],
							},
						},
					}, {
						'staking': {
							'delegate': {
								'validator': 'starsvaloper1rd6wzd9kwsg4fgdew2xs842rrqsdl3jdlwsapl',
								'amount': { 'denom': 'ustars', 'amount': '500000' },
							},
						},
					}],
				},
			},
			'signature':
				'CLPfmTNY8/ATInPCAUyvmqNZlnMTb1nlPLsdjt5lBq9jkN0faQV7H6/Pqf7fiKdZaVdiLnLCiPJqwgKe3VM9Sxw=',
			signer,
		};

		const msg: ExecuteAccountMsg = { execute: eth_typed_data.message.execute };
		await executeSignedAction(chain, msg, { eth_typed_data }, [{
			denom: 'ustars',
			amount: '1000000',
		}]);

		expect(chain.address.length).toBeGreaterThan(0);
	}); */

	test('Eth typed transfer nft', async () => {
		console.log('Running full setup...');

		const cred_acc = chain.contracts.cw82_tba_credentials.address;
		expect(cred_acc).toBeDefined();

		await mintToken(chain, collection, '3');

		const { owner } = await queryTokenOwner(chain, '3');
		if (owner == chain.address) {
			await sendToken(chain, cred_acc, '3');
			await sleep(1500);
		}

		const info: FullInfoResponse = await chain.queryClient.wasm.queryContractSmart(cred_acc, {
			full_info: {},
		});

		const found = info.tokens.find((t) => t.id == '3' && t.collection == collection);
		expect(found).toBeDefined();

		const { owner: new_owner } = await queryTokenOwner(chain, '3');
		expect(new_owner).toBe(cred_acc);

		const nonce = info.credentials.account_number;

		if (nonce <= 1) await sleep(2000);
		else if (nonce != 2) return;

		const eth_typed_data = {
			'types': {
				'EIP712Domain': [
					{ 'name': 'name', 'type': 'string' },
					{ 'name': 'version', 'type': 'string' },
					{ 'name': 'chainId', 'type': 'uint256' },
					{ 'name': 'verifyingContract', 'type': 'address' },
				],
				'Transfer': [{ 'name': 'collection', 'type': 'string' }, {
					'name': 'recipient',
					'type': 'string',
				}, { 'name': 'token_id', 'type': 'string' }],
				'AccountAction': [{ 'name': 'transfer_token', 'type': 'Transfer' }],
			},
			'primaryType': 'AccountAction',
			'domain': {
				'verifyingContract': '0x9774c39fd649c03c114e8cfa4d42e5ca38e13b4f',
				'name': 'Token-Bound Accounts',
				'version': '1.1',
				'chainId': '0',
			},
			'message': {
				'transfer_token': {
					'collection': 'stars1wkwy0xh89ksdgj9hr347dyd2dw7zesmtrue6kfzyml4vdtz6e5ws2hcm9v',
					'recipient': 'stars16z43tjws3vw06ej9v7nrszu0ldsmn0eyjnjpu8',
					'token_id': '3',
				},
			},
			'signature':
				'pP6tRxdIi/9lRMUGmMMDa0FPqfjab+iImxU3aWj6fCJjD8PXdCrO4EwdxRTUaB6CvHs9ijYYDhh6PH3m+kf1LRs=',
			signer,
		};

		const msg: ExecuteAccountMsg = {
			transfer_token: { collection, recipient: chain.address, token_id: '3' },
		};
		await executeSignedAction(chain, msg, { eth_typed_data });

		expect(chain.address.length).toBeGreaterThan(0);
	});
});
