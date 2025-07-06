import { readFileSync, writeFileSync } from 'fs';
import type { ChainConfig, ContractConfig } from './types';

const CONFIG_FOLDER = 'configs';
const CHAIN_CONFIG_NAME = 'chain';
const CONTRACT_CONFIG_NAME = 'contracts';
const CHAIN_CONFIG_PATH = `${CONFIG_FOLDER}/${CHAIN_CONFIG_NAME}.json`;
const CONTRACT_CONFIG_PATH = `${CONFIG_FOLDER}/${CONTRACT_CONFIG_NAME}.json`;

export const getChainConfig = (): ChainConfig => {
	return JSON.parse(readFileSync(CHAIN_CONFIG_PATH, 'utf8')) as ChainConfig;
};

export const saveChainConfig = (config: ChainConfig): void => {
	writeFileSync(CHAIN_CONFIG_PATH, JSON.stringify(config, null, 2));
};

export const getContractConfig = (): ContractConfig => {
	return JSON.parse(readFileSync(CONTRACT_CONFIG_PATH, 'utf8'));
};

export const saveContractConfig = (config: ContractConfig): void => {
	writeFileSync(CONTRACT_CONFIG_PATH, JSON.stringify(config, null, 2));
};
