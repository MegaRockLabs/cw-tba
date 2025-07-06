/* import type { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import fs from 'fs';

import { getContractConfig, saveContractConfig } from './config';
import { uploadContract } from './contract';
import type { ChainData, Contract, ContractConfig, ContractName } from './types';

const ARTIFACTS_FOLDER = 'artifacts';

// Dynamically get contract names from wasm files in artifacts folder
export const getContractNames = (): ContractName[] => {
	try {
		const files = fs.readdirSync(ARTIFACTS_FOLDER);
		const wasmFiles = files.filter(file => file.endsWith('.wasm'));

		// Remove extension and architecture suffix (e.g., "-aarch64")
		const contractNames = wasmFiles.map(file => {
			let name = file.replace('.wasm', '');
			// Remove architecture suffixes like -aarch64, -x86_64, etc.
			name = name.replace(/-aarch64$/, '').replace(/-x86_64$/, '').replace(/-arm64$/, '');
			return name as ContractName;
		});
		// Remove duplicates
		return [...new Set(contractNames)];
	} catch (error) {
		console.error('Error reading artifacts folder:', error);
		return [];
	}
};

// Get list of contracts that should be uploaded
export const contractsToUpload = (): ContractName[] => {
	return getContractNames();
};

// Additional environment variable constants
export const ENV_DEFAULTS = {
	RPC_ENDPOINT: 'http://localhost:26657',
	GRPC_ENDPOINT: 'http://localhost:9090',
	CHAIN_ID: 'testing',
	PREFIX: 'stars',
	DENOM: 'ustars',
	GAS_PRICE: '0.025',
	GAS_ADJUSTMENT: '1.5',
	DERIVATION_PATH: "m/44'/118'/0'/0/0",
	MNEMONIC:
		'game cook seven draw girl special once poem rhythm seven winner praise demise trick access style bracket later tunnel slush lab false game park',
} as const;

// Validate environment setup with detailed reporting
export const validateEnvironment = (): {
	isValid: boolean;
	errors: string[];
	warnings: string[];
} => {
	const errors: string[] = [];
	const warnings: string[] = [];

	// Required variables
	const required = ['RPC_ENDPOINT', 'CHAIN_ID', 'PREFIX', 'DENOM'];
	required.forEach(key => {
		if (!process.env[key]) {
			errors.push(`Missing required environment variable: ${key}`);
		}
	});

	// Optional but recommended variables
	const recommended = ['GRPC_ENDPOINT', 'GAS_PRICE', 'GAS_ADJUSTMENT'];
	recommended.forEach(key => {
		if (!process.env[key]) {
			warnings.push(`Missing optional environment variable: ${key} (will use default)`);
		}
	});

	// Validate values
	if (process.env.GAS_PRICE && isNaN(parseFloat(process.env.GAS_PRICE))) {
		errors.push('GAS_PRICE must be a valid number');
	}

	if (process.env.GAS_ADJUSTMENT && isNaN(parseFloat(process.env.GAS_ADJUSTMENT))) {
		errors.push('GAS_ADJUSTMENT must be a valid number');
	}

	return { isValid: errors.length === 0, errors, warnings };
};

// Check if all required environment variables are set
export const checkedEnv = async () => {
	const validation = validateEnvironment();

	if (!validation.isValid) {
		console.error('❌ Environment validation failed:');
		validation.errors.forEach(error => console.error(`  - ${error}`));
		throw new Error(`Environment validation failed: ${validation.errors.join(', ')}`);
	}

	if (validation.warnings.length > 0) {
		console.warn('⚠ Environment warnings:');
		validation.warnings.forEach(warning => console.warn(`  - ${warning}`));
	}

	// Set default values for optional environment variables
	Object.entries(ENV_DEFAULTS).forEach(([key, defaultValue]) => {
		if (!process.env[key]) {
			process.env[key] = defaultValue;
		}
	});

	console.log('✓ Environment variables checked and defaults applied');
};

// Upload a new version of the contract and instantiate it
// Updates code_id and address of the contract object
export const updatedContract = async (
	client: SigningCosmWasmClient,
	sender: string,
	contract: Contract,
): Promise<Contract> => {
	try {
		console.log(`Uploading contract: ${contract.name}`);
		const codeId = await uploadContract(client, sender, contract.name);
		contract.code_id = codeId;
		console.log(`✓ Uploaded ${contract.name} with code ID: ${codeId}`);
	} catch (error) {
		throw new Error(`Error uploading ${contract.name} contract: ${error}`);
	}

	return await updatedContractAddress(client, sender, contract);
};

// Instantiate the contract using the code_id
// Updates address of the contract object
export const updatedContractAddress = async (
	client: SigningCosmWasmClient,
	sender: string,
	contract: Contract,
): Promise<Contract> => {
	try {
		console.log(`Instantiating contract: ${contract.name}`);
		const address = await instantiateContract(client, sender, contract.code_id, contract.name);
		console.log(`✓ Instantiated ${contract.name} at address: ${address}`);

		return { ...contract, address };
	} catch (error) {
		throw new Error(`Error instantiating ${contract.name} contract: ${error}`);
	}
};

// Upload and instantiate all known contracts
export const initContracts = async (
	client: SigningCosmWasmClient,
	sender: string,
): Promise<Contract[]> => {
	const contracts: Contract[] = [];
	const contractNames = contractsToUpload();

	console.log(`Initializing ${contractNames.length} contracts...`);

	for (const name of contractNames) {
		try {
			const contract = await updatedContract(client, sender, { name, code_id: 0 });
			contracts.push(contract);
		} catch (error) {
			console.error(`Failed to initialize contract ${name}:`, error);
			throw error;
		}
	}
	console.log(`✓ All contracts initialized successfully`);
	return contracts;
};

// Check if contracts are properly deployed on chain
export const checkedChainContracts = async (
	client: SigningCosmWasmClient,
	sender: string,
	contracts: Contract[],
): Promise<{ updated: boolean; contracts: Contract[] }> => {
	const codes = await client.getCodes();
	const checkedContracts: Contract[] = [];
	let updated = false;

	const mustContracts = contractsToUpload();
	console.log(`Checking ${mustContracts.length} required contracts...`);

	// Check for missing contracts
	for (const name of mustContracts) {
		const existingContract = contracts.find(x => x.name === name);
		if (!existingContract) {
			console.log(`Contract ${name} not found. Uploading...`);
			updated = true;
			checkedContracts.push(await updatedContract(client, sender, { name, code_id: 0 }));
		}
	}

	// Check existing contracts
	for (const contract of contracts) {
		// Handle contract migration if needed
		if (contract.migrate && contract.address) {
			console.log(`Migrating contract: ${contract.name}`);
			const codeId = await uploadContract(client, sender, contract.name);
			contract.code_id = codeId;
			contract.migrate = false;

			await client.migrate(sender, contract.address, contract.code_id, {}, 'auto');
			checkedContracts.push(contract);
			updated = true;
		} // Check if code ID exists on chain
		else if (!codes.find(x => x.id === contract.code_id)) {
			console.warn(`Code ID for ${contract.name} not found on chain. Re-uploading...`);
			updated = true;
			checkedContracts.push(await updatedContract(client, sender, contract));
		} // Check if contract is instantiated
		else {
			const instantiatedContracts = await client.getContracts(contract.code_id);
			const isInstantiated = contract.address &&
				instantiatedContracts.some(addr => addr === contract.address);

			if (isInstantiated) {
				checkedContracts.push(contract);
			} else {
				console.warn(
					`Contract address for ${contract.name} not found on chain. Re-instantiating...`,
				);
				updated = true;
				checkedContracts.push(await updatedContractAddress(client, sender, contract));
			}
		}
	}

	// Ensure all required contracts are present
	for (const name of mustContracts) {
		if (!checkedContracts.find(x => x.name === name)) {
			console.warn(`Contract ${name} missing. Uploading...`);
			updated = true;
			checkedContracts.push(await updatedContract(client, sender, { name, code_id: 0 }));
		}
	}

	if (updated) {
		console.log('✓ Contract deployment updated');
	} else {
		console.log('✓ All contracts are up to date');
	}

	return { updated, contracts: checkedContracts };
};

// Check and deploy contracts, updating configuration as needed
export const checkedContracts = async (data: ChainData): Promise<ContractConfig> => {
	const config = getContractConfig();

	console.log('Checking contract deployment...');

	let contracts: Contract[] = [];

	// Load existing contracts from config or initialize new ones
	if (config.contract_info && config.contract_info.contracts) {
		contracts = config.contract_info.contracts;
		console.log(`Loaded ${contracts.length} existing contracts from config`);
	} else {
		console.log('No existing contract configuration found, initializing new deployment');
	}

	// Check and update contracts on chain
	const { updated, contracts: checkedContracts } = await checkedChainContracts(
		data.client,
		data.account,
		contracts,
	);

	// Update config if changes were made
	if (updated || !config.contract_info) {
		const updatedConfig: ContractConfig = {
			...config,
			contract_info: {
				contracts: checkedContracts,
				deployment_timestamp: new Date().toISOString(),
				chain_id: process.env.CHAIN_ID || 'testing',
				deployer_address: data.account,
			},
		};

		saveContractConfig(updatedConfig);
		console.log('✓ Contract configuration updated and saved');

		// Log deployment summary
		console.log('\n📋 Contract Deployment Summary:');
		checkedContracts.forEach(contract => {
			console.log(`  • ${contract.name}:`);
			console.log(`    - Code ID: ${contract.code_id}`);
			console.log(`    - Address: ${contract.address || 'Not instantiated'}`);
		});
		console.log('');

		return updatedConfig;
	}

	console.log('✓ All contracts are up to date');
	return config;
};

// Main setup function that checks environment, contracts, and returns chain data
export const checkedSetup = async (): Promise<ChainData> => {
	console.log('Starting chain setup...');

	// Check environment variables
	await checkedEnv();

	// Initialize chain connection
	const { getChainData: initChain } = await import('./chain');
	const chainData = await initChain();

	// Check and deploy contracts
	await checkedContracts(chainData);

	console.log('✓ Chain setup completed successfully');
	return chainData;
};

// Helper function to get contract address by name
export const getContractAddress = (config: ContractConfig, name: ContractName): string => {
	const contract = config.contract_info?.contracts.find(c => c.name === name);
	if (!contract?.address) {
		throw new Error(`Contract address not found for: ${name}`);
	}
	return contract.address;
};

// Helper function to get contract code ID by name
export const getContractCodeId = (config: ContractConfig, name: ContractName): number => {
	const contract = config.contract_info?.contracts.find(c => c.name === name);
	if (!contract?.code_id) {
		throw new Error(`Contract code ID not found for: ${name}`);
	}
	return contract.code_id;
};

// Helper function to get all contract information
export const getAllContractInfo = (config: ContractConfig): Contract[] => {
	return config.contract_info?.contracts || [];
};

// Helper function to validate contract deployment
export const validateContractDeployment = (config: ContractConfig): boolean => {
	const contracts = getAllContractInfo(config);
	const requiredContracts = contractsToUpload();

	// Check if all required contracts are present
	for (const requiredName of requiredContracts) {
		const contract = contracts.find(c => c.name === requiredName);
		if (!contract) {
			console.warn(`Missing required contract: ${requiredName}`);
			return false;
		}

		if (!contract.code_id || contract.code_id <= 0) {
			console.warn(`Invalid code ID for contract: ${requiredName}`);
			return false;
		}

		// Note: Some contracts (like TBA contracts) may not have addresses initially
		// as they're instantiated through the registry
		if (!contract.address && !['cw82_tba_base', 'cw82_tba_credentials'].includes(contract.name)) {
			console.warn(`Missing address for contract: ${requiredName}`);
			return false;
		}
	}

	return true;
};

// Helper function to export contract configuration for external tools
export const exportContractConfig = (config: ContractConfig): Record<string, any> => {
	const contracts = getAllContractInfo(config);
	const exportData: Record<string, any> = {
		metadata: {
			deployment_timestamp: config.contract_info?.deployment_timestamp,
			chain_id: config.contract_info?.chain_id,
			deployer_address: config.contract_info?.deployer_address,
			total_contracts: contracts.length,
		},
		contracts: {},
	};

	contracts.forEach(contract => {
		exportData.contracts[contract.name] = {
			code_id: contract.code_id,
			address: contract.address,
			migrate: contract.migrate || false,
		};
	});

	return exportData;
};
 */
