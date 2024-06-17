#!/bin/bash

BINARY=${BINARY:-archwayd}
FOLDER=${FOLDER:-archway}
CHAINID=${CHAINID:-testing}
DENOM=${DENOM:-aarch}
BLOCK_GAS_LIMIT=${GAS_LIMIT:-75000000000}

IAVL_CACHE_SIZE=${IAVL_CACHE_SIZE:-1562500}
QUERY_GAS_LIMIT=${QUERY_GAS_LIMIT:-50000000}
SIMULATION_GAS_LIMIT=${SIMULATION_GAS_LIMIT:-5000000000}
MEMORY_CACHE_SIZE=${MEMORY_CACHE_SIZE:-1000}

# Build genesis file incl account for each address passed in
coins="10000000000000000000000$DENOM"
$BINARY init --chain-id $CHAINID $CHAINID
$BINARY keys add validator --keyring-backend="test"
$BINARY genesis add-genesis-account validator $coins --keyring-backend="test"

# create account for each passed in address
for addr in "$@"; do
  echo "creating genesis account: $addr"
  $BINARY genesis add-genesis-account $addr $coins --keyring-backend="test"
done

$BINARY genesis gentx validator 10000000000000000000000$DENOM --chain-id $CHAINID --keyring-backend="test"
$BINARY genesis collect-gentxs



# Set proper defaults and change ports
sed -i 's/"localhost:9090"/"0.0.0.0:9090"/g' ~/.$FOLDER/config/app.toml
sed -i 's/"localhost:9091"/"0.0.0.0:9091"/g' ~/.$FOLDER/config/app.toml

sed -i 's/"leveldb"/"goleveldb"/g' ~/.$FOLDER/config/config.toml
sed -i 's#"tcp://127.0.0.1:26657"#"tcp://0.0.0.0:26657"#g' ~/.$FOLDER/config/config.toml
sed -i "s/\"stake\"/\"$DENOM\"/g" ~/.$FOLDER/config/genesis.json
sed -i "s/\"max_gas\": \"-1\"/\"max_gas\": \"$BLOCK_GAS_LIMIT\"/" ~/.$FOLDER/config/genesis.json
sed -i 's/timeout_commit = "5s"/timeout_commit = "1s"/g' ~/.$FOLDER/config/config.toml
sed -i 's/timeout_propose = "3s"/timeout_propose = "1s"/g' ~/.$FOLDER/config/config.toml
sed -i 's/index_all_keys = false/index_all_keys = true/g' ~/.$FOLDER/config/config.toml

sed -i "s/iavl-cache-size = 781250/iavl-cache-size = $IAVL_CACHE_SIZE/g" ~/.$FOLDER/config/app.toml
sed -i "s/query_gas_limit = 50000000/query_gas_limit = $QUERY_GAS_LIMIT/g" ~/.$FOLDER/config/app.toml
sed -i "s/simulation_gas_limit = 25000000/simulation_gas_limit = $SIMULATION_GAS_LIMIT/g" ~/.$FOLDER/config/app.toml
sed -i "s/memory_cache_size = 512/memory_cache_size = $MEMORY_CACHE_SIZE/g" ~/.$FOLDER/config/app.toml

# Start the stake
$BINARY start --pruning=nothing --minimum-gas-prices 0.1$DENOM 