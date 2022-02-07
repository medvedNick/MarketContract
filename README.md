# Market Contract 

Simple Rust market contract for token exchange.

## Description

There are two contracts - `market` and `token_holder` (one per token). The first one acts as an exchange and contains the business logic, while the last ones simply store the tokens with the ability to transfer them with the command from the market contract. You can add as many tokens as you want, while exchanging between any of them.

The main flow is as follows:

1. Deploy contract (for example, to `market.medvednick.testnet`)
2. Add tokens with appropriate balance
    - use `add_token` method with parameter `{"token": <TOKEN_NAME>}`
    - minumum deposit is 0.02 Ⓝ
3. Contract creates sub-account (i.e. `token1.market.medvednick.testnet`) and deploys the `token_holder` contract to it
3. Execute market orders between any tokens with automatic rate as X * Y = K
    - use `execute_order` method with parameter 
        ```
        {
            "sell": <TOKEN_YOU_WANT_TO_SELL>,
            "buy": <TOKEN_YOU_WANT_TO_BUY,
            "token_id": <WHERE_TO_TRANSFER_BUY_TOKEN>
        }
        ```
    - minumum deposit is 0.01 Ⓝ
4. You can remove also remove token from market which will also delete token subaccount
    - use `remove_token` with parameter `{"token": <TOKEN_NAME>}`

## Unit Tests

You can run unit tests from root folder with:

`./test.sh`

or for each contract from its directory with the same command.

## Deploy and Test Run

You'll need near cli and other tools to build and deploy the contract.

You can deploy and run automatic testing the contract on **TESTNET** with script with:

`./deploy.sh <TESTNET_ACC_WUTHOUT_SUFFIX>`

For example:
    
`./deploy.sh medvednick`

will deploy contract to `market.medvednick.testnet` using `medvednick.testnet` as master account. The script will first try to delete current market.* account and then to add 2 tokens to market sending 2 Near to each. Tokens would be added into `<TOKEN>.market.medvednick.testnet` accounts. The last step will exchange 0.25 Near between tokens.

You can look into `deploy.sh` and comment/modify any commands.

## Thoughts on bugs

There are multiple bugs and possible vulnerabilities within the contract, but due to the test nature of the task they are not fixed. Although they are mentioned in the list below, to show the awareness of them.

- Remove token does not work properly because of some errors with ownership, need to find out why
- Not sure if it's good to use `cd` in build scripts, but it's not working from other directory due to `.toml` absence
- Fees and gas calculation probably should be made more dynamic
- Probably it's also worth to move fees into class variables to be able to override them inside tests
- No checks for success of `Promise`s, must-have but it will require a lot of additional code :(
- K = X * Y results in overflow, changing formulae allows to avoid it, but still we have accuracy losses
- Not sure how to unit test returned `Promise`s, can we?
- Function `new::` on `token_holder` may be changed to `function_call`. Should check if there is any difference
- Deploying contract in two different calls (`deploy` and call `new`) is a bad practice

