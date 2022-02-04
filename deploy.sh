#!/bin/bash
set -e

./build.sh

near delete market.$1.testnet $1.testnet || true
near create-account market.$1.testnet --masterAccount $1.testnet
near deploy market.$1.testnet --wasmFile res/market_contract.wasm
near call market.$1.testnet new '{"owner_id": "'${1}'.testnet"}' --accountId $1.testnet
near call market.$1.testnet add_token '{"token": "token1"}' --amount 1 --accountId $1.testnet
near call market.$1.testnet add_token '{"token": "token2"}' --amount 1 --accountId $1.testnet
near call market.$1.testnet add_token '{"token": "token3"}' --amount 1 --accountId $1.testnet

# Uncomment this for executing order
# near call market.$1.testnet execute_order '{"sell": "token1", "buy": "token2"}' --amount 0.25 --accountId $1.testnet
