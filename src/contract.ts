import { type SigningCosmWasmClient, toBinary } from '@cosmjs/cosmwasm-stargate';
import type { Coin } from '@cosmjs/stargate';
import { existsSync, readFileSync } from 'fs';
import { getContractConfig, saveContractConfig } from './config';
import {
	type AccountResponseForTokenInfo,
	type ChainData,
	type ContractName,
	CREATION_FB_FEE,
	type Credential,
	type CredentialData,
	type ExecuteAccountMsg,
	type ExecuteMsg,
	type OwnerOfResponse,
	type QueryMsg,
	type RegistryMsg,
	type SignedDataMsg,
} from './types';

const CONTRACT_FOLDER = 'artifacts/';

export const CONTRACT_NAMES: ContractName[] = [
	'cw721_base',
	'cw82_tba_base',
	'cw82_tba_credentials',
	'cw83_tba_registry',
];

// Get file path for contract wasm file
export const nameToFilePath = (contractName: ContractName) => {
	// Try exact match first
	let filePath = CONTRACT_FOLDER + contractName + '.wasm';
	if (existsSync(filePath)) {
		return filePath;
	}

	// Try with architecture suffix
	filePath = CONTRACT_FOLDER + contractName + '-aarch64.wasm';
	if (existsSync(filePath)) {
		return filePath;
	}

	throw new Error(`Contract file not found for ${contractName}`);
};

// Upload contract to chain
export const uploadContract = async (
	client: SigningCosmWasmClient,
	sender: string,
	name: ContractName,
): Promise<number> => {
	const wasmPath = nameToFilePath(name);

	if (!existsSync(wasmPath)) {
		throw new Error(`${name} contract file not found at ${wasmPath}`);
	}

	console.log(`Uploading contract ${name}.wasm...`);

	const wasmByteCode = new Uint8Array(readFileSync(wasmPath));

	try {
		const result = await client.upload(sender, wasmByteCode, 'auto');
		console.log(
			`✓ Successfully uploaded ${name} with code ID ${result.codeId}. Gas used: ${result.gasUsed}`,
		);
		return result.codeId;
	} catch (error) {
		throw new Error(`Error uploading ${name} contract: ${error}`);
	}
};

// Execute contract message
export const executeContract = async (
	client: SigningCosmWasmClient,
	sender: string,
	contractAddress: string,
	msg: any,
	funds: any[] = [],
): Promise<any> => {
	try {
		const result = await client.execute(sender, contractAddress, msg, 'auto', undefined, funds);
		return result;
	} catch (error) {
		throw new Error(`Error executing contract: ${error}`);
	}
};

// Query contract
export const queryContract = async (
	client: SigningCosmWasmClient,
	contractAddress: string,
	queryMsg: any,
): Promise<any> => {
	try {
		const result = await client.queryContractSmart(contractAddress, queryMsg);
		return result;
	} catch (error) {
		throw new Error(`Error querying contract: ${error}`);
	}
};

export const instantiateRegistry = async (chain: ChainData) => {
	const c = chain.contracts;

	const instantiateMsg = {
		params: {
			allowed_code_ids: [c.cw82_tba_base.code_id, c.cw82_tba_credentials.code_id],
			creation_fees: [{ denom: chain.config.denom, amount: CREATION_FB_FEE }],
			managers: [],
			extension: {},
		},
	};

	const result = await chain.client.instantiate(
		chain.address,
		c.cw83_tba_registry.code_id,
		instantiateMsg,
		'registry',
		'auto',
	);
	console.log(
		`✓ Successfully instantiated TBA registry at ${result.contractAddress}. Gas used: ${result.gasUsed}`,
	);

	chain.contracts.cw83_tba_registry.address = result.contractAddress;
	saveContractConfig(chain.contracts);

	return result.contractAddress;
};

export const instantiateCollection = async (chain: ChainData, minter?: string) => {
	const result = await chain.client.instantiate(
		chain.address,
		chain.contracts.cw721_base.code_id,
		{ name: 'test', symbol: 'test', minter: minter || chain.address },
		'collection',
		'auto',
	);

	chain.contracts.cw721_base.address = result.contractAddress;
	saveContractConfig(chain.contracts);

	return result.contractAddress;
};

export const mintToken = async (chain: ChainData, collection: string, token_id: string) => {
	const { owner } = await queryTokenOwner(chain, token_id);
	if (!owner) {
		const mintMsg = { mint: { token_id, owner: chain.address, token_uri: null, extension: null } };
		await chain.client.execute(chain.address, collection, mintMsg, 'auto');
	}
};

export const createAccount = async (
	chain: ChainData,
	collection: string,
	id: string,
	credential_data: CredentialData,
	base: boolean = true,
	save: boolean = true,
) => {
	const info = await queryAccountInfo(chain, collection, id);
	if (info) {
		console.log('✓ TBA account already exists for this token:', info);
		if (save) {
			if (base) chain.contracts.cw82_tba_base.address = info.address;
			else chain.contracts.cw82_tba_credentials.address = info.address;
			saveContractConfig(chain.contracts);
		}
		return info.address;
	}

	const code_id = base
		? chain.contracts.cw82_tba_base.code_id
		: chain.contracts.cw82_tba_credentials.code_id;

	const createAccountMsg: RegistryMsg = {
		create_account: {
			code_id,
			chain_id: chain.config.chain_id,
			account_data: { token_info: { collection, id }, credential_data },
		},
	};
	const result = await chain.client.execute(
		chain.address,
		chain.contracts.cw83_tba_registry.address!,
		createAccountMsg,
		'auto',
		undefined,
		[{ denom: chain.config.denom, amount: CREATION_FB_FEE }],
	);

	// console.log(`✓ Successfully created TBA account for ${collection}:${id} at ${result}. Gas used: ${result.gasUsed}`);
	// console.log('Result:', result);
	// chain.contracts.cw82_tba_base.address = result.contractAddress;
	const event = result.events.find(e => e.type === 'instantiate');
	const tokenAccountAddress = event?.attributes.find(a => a.key === '_contract_address')?.value;

	if (save && tokenAccountAddress) {
		chain.contracts.cw82_tba_base.address = tokenAccountAddress;
		saveContractConfig(chain.contracts);
	}

	return tokenAccountAddress;
};

export const transferToken = async (chain: ChainData, recipient: string, token_id: string) => {
	return await chain.client.execute(chain.address, chain.contracts.cw721_base.address!, {
		transfer_nft: { recipient, token_id },
	}, 'auto');
};

export const sendToken = async (chain: ChainData, contract: string, token_id: string) => {
	return await chain.client.execute(chain.address, chain.contracts.cw721_base.address!, {
		send_nft: { contract, token_id, msg: '' },
	}, 'auto');
};

export const executeSignedAction = async (
	chain: ChainData,
	action: ExecuteAccountMsg,
	cred: Credential,
	funds: Coin[] = [],
) => {
	const contract = chain.contracts.cw82_tba_credentials.address;
	const executeMsg: ExecuteMsg = { execute_signed: { msg: action, signed: cred } };
	try {
		return await chain.client.execute(
			chain.address,
			contract,
			executeMsg,
			'auto',
			undefined,
			funds,
		);
	} catch (error) {
		throw new Error(`Error executing action: ${error}`);
	}
};

export const queryTokenOwner = async (
	chain: ChainData,
	token_id: string,
): Promise<OwnerOfResponse> => {
	const address = chain.contracts.cw721_base.address;
	const queryMsg = { owner_of: { token_id } };
	try {
		return await chain.client.queryContractSmart(address, queryMsg);
	} catch (error) {
		// console.error(`Error querying token owner for ${token_id}:`, error);
		return { owner: '' };
	}
};

export const queryAccountInfo = async (
	chain: ChainData,
	collection: string,
	id: string,
): Promise<AccountResponseForTokenInfo | null> => {
	const address = chain.contracts.cw83_tba_registry.address;
	const queryMsg: QueryMsg = { account_info: { collection, id } };
	try {
		return await chain.client.queryContractSmart(address, queryMsg);
	} catch (error) {
		// console.error(`Error querying account info for ${id}:`, error);
		return null;
	}
};
