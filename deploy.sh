#!/bin/bash
set -e

if [ "$#" -ne 1 ]; then
  echo "Usage: $0 <ACCOUNT_ID>" >&2
  echo "example: $0 medvednick" >&2
  exit 1
fi

near delete market.$1.testnet $1.testnet || true
near create-account market.$1.testnet --masterAccount $1.testnet
near deploy market.$1.testnet --wasmFile market/res/market_contract.wasm
near call market.$1.testnet new '{"owner_id": "'${1}'.testnet"}' --accountId $1.testnet

# Comment this for stop adding tokens on deploy
near call market.$1.testnet add_token '{"token": "token1"}' --amount 2 --accountId $1.testnet
near call market.$1.testnet add_token '{"token": "token2"}' --amount 2 --accountId $1.testnet

# Comment this for stop executing order on deploy
near call market.$1.testnet execute_order '{"sell": "token1", "buy": "token2", "token_id": "'${1}'.testnet"}' --amount 0.25 --accountId $1.testnet
