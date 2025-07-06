starsd tx wasm store artifacts/cw82_tba_credentials.wasm --from local  -y --gas-prices 0.1ustars  --gas 5500000 --node http://localhost:26657  --chain-id testing
sleep 3
starsd tx wasm store artifacts/cw83_tba_registry.wasm --from local  -y --gas-prices 0.1ustars  --gas 5500000 --node http://localhost:26657  --chain-id testing
sleep 3
starsd tx wasm store artifacts/cw82_tba_base.wasm --from local  -y --gas-prices 0.1ustars  --gas 5500000 --node http://localhost:26657  --chain-id testing
sleep 3
starsd tx wasm store artifacts/cw721_base-aarch64.wasm --from local  -y --gas-prices 0.1ustars  --gas 5500000 --node http://localhost:26657  --chain-id testing

echo '{"cw721_base":{"code_id":4},"cw82_tba_base":{"code_id":3},"cw83_tba_registry":{"code_id":2},"cw82_tba_credentials":{"code_id":1}}' > configs/contracts.json
