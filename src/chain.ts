import { Secp256k1HdWallet } from '@cosmjs/amino';
import { setupWasmExtension, SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { wasmTypes } from '@cosmjs/cosmwasm-stargate';
import { Decimal } from '@cosmjs/math';
import { DirectSecp256k1Wallet, type GeneratedType, Registry } from '@cosmjs/proto-signing';

import { defaultRegistryTypes, QueryClient, setupBankExtension } from '@cosmjs/stargate';
import { Comet38Client } from '@cosmjs/tendermint-rpc';
import { readFileSync } from 'fs';
import { getChainConfig, getContractConfig } from './config';
import { initialSetup } from './setup';
import type { Account, ChainConfig, ChainData, ChainQueryClient } from './types';

const CONNECT_INTERVAL = 3000;
const CONNECT_MAX_ATTEMPTS = 10;

const accountsPath = 'configs/accounts.json';
const registryTypes: [string, GeneratedType][] = [...defaultRegistryTypes, ...wasmTypes];

// Chain connection state
export let wallet: Secp256k1HdWallet | undefined = undefined;
export let account: string | undefined = undefined;
export let signingClient: SigningCosmWasmClient | undefined = undefined;
export let queryClient: ChainQueryClient | undefined = undefined;

export const getClient = (): SigningCosmWasmClient => {
	if (!signingClient) {
		throw new Error('Signing client not initialized. Call initChain() first.');
	}
	return signingClient;
};

export const getQueryClient = (): ChainQueryClient => {
	if (!queryClient) {
		throw new Error('Query client not initialized. Call initChain() first.');
	}
	return queryClient;
};

const loadTestAccounts = (): Account[] => {
	try {
		const accountsData = readFileSync(accountsPath, 'utf8');
		const accs = JSON.parse(accountsData);
		account = accs[0].address;
		return accs as Account[];
	} catch (error) {
		console.error('Error loading test accounts:', error);
		throw new Error('Failed to load test accounts from configs/test_accounts.json');
	}
};

// Gas price helper
const createGasPrice = (denom: string, amount?: string) => ({
	amount: Decimal.fromUserInput(amount ?? '0.025', 100),
	denom,
});

export const getAccount = async (): Promise<string> => {
	if (account) return account;
	if (wallet) {
		const accounts = await wallet.getAccounts();
		account = accounts[0].address;
		return account;
	}
	throw new Error('Wallet not initialized');
};

export const getWallet = async (
	config: ChainConfig,
	accounts: Account[],
	accountIndex: number = 0,
): Promise<Secp256k1HdWallet> => {
	if (!wallet || !account) {
		const selectedAccount = accounts[accountIndex];
		if (!selectedAccount) {
			throw new Error(`Account at index ${accountIndex} not found`);
		}

		wallet = await Secp256k1HdWallet.fromMnemonic(selectedAccount.mnemonic, {
			prefix: config.prefix,
		});
		account = (await wallet.getAccounts())[0].address;
	}
	return wallet;
};

export const getClients = async (
	config: ChainConfig,
	wallet: Secp256k1HdWallet,
	attempt: number = 1,
): Promise<{ client: SigningCosmWasmClient; queryClient: ChainQueryClient }> => {
	if (!signingClient || !queryClient) {
		try {
			const gasPrice = createGasPrice(config.denom, config.gas_price.toString());

			const cometClient = await Comet38Client.connect(config.rpc_endpoint);

			signingClient = await SigningCosmWasmClient.createWithSigner(cometClient, wallet, {
				gasPrice,
				registry: new Registry(registryTypes),
			});

			queryClient = QueryClient.withExtensions(cometClient, setupWasmExtension, setupBankExtension);
		} catch (error) {
			if (attempt >= CONNECT_MAX_ATTEMPTS) {
				throw new Error('Max connection attempts reached. Could not connect to the chain');
			}
			await new Promise(resolve => setTimeout(resolve, CONNECT_INTERVAL));
			return await getClients(config, wallet, attempt + 1);
		}
	}
	return { client: signingClient, queryClient: queryClient! };
};

export const getChainData = async (accountIndex: number = 0): Promise<ChainData> => {
	const config = getChainConfig();
	const contracts = getContractConfig();
	const accounts = loadTestAccounts();
	const address = await getAccount();
	const wallet = await getWallet(config, accounts, accountIndex);
	const { client, queryClient } = await getClients(config, wallet);

	const chain = { wallet, address, queryClient, contracts, client, config, accounts };
	await initialSetup(chain);
	return chain;
};
