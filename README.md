# Market Contract 
by @medvedinck

## Description

Simple Rust market contract, which can be used as follows:

1. Deploy contract
2. Add tokens (token name is choosen to be AccountId for future improvements) with appropriate balance
3. Execute market orders with automatic rate as X * Y = K

## Unit Tests

You can run unit tests with:
    `./test.sh`

## Deploy

You'll need near cli and other tools to build and deploy the contract.

You can deploy the contract with script with:
    `./deploy.sh <TESTNET_ACC_WUTHOUT_SUFFIX>`

For example:
    `./deploy.sh medvednick`
will deploy contract to `market.medvednick.testnet` using `medvednick.testnet` as master account.
The script will first try to delete current market.* account and then to add 3 tokens to market sending 1 Near to each.

## Running

Check the deploy script for examples.

You can add token to the market with:
`near call market.youraccount.testnet add_token '{"token": "token1"}' --amount 1 --accountId youraccount.testnet`

You can exchange tokens with (not requiring the sender account to be the master one):
`near call market.youraccount.testnet execute_order '{"sell": "token1", "buy": "token2"}' --amount 0.25 --accountId youraccount.testnet`


