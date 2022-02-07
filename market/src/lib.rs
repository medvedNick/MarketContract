use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::{
    env, near_bindgen, ext_contract, require,
    AccountId, PanicOnDefault, Promise, Gas, Balance
};

const TOKEN_CONTRACT: &[u8] = include_bytes!("../../token/res/token_contract.wasm");

const ADD_TOKEN_RESERVE: Balance = 10_000_000_000_000_000_000_000; // 0.01 Ⓝ, reserve to store on main contract when adding token
const TOKEN_RESERVE: Balance = 10_000_000_000_000_000_000_000; // 0.01 Ⓝ, reserve to store on token contract
const EXCHANGE_TOKEN_FEE: Balance = 10_000_000_000_000_000_000_000; // 0.01 Ⓝ, exchange fee
const BASE_GAS: Gas = Gas(5_000_000_000_000);

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Contract {
    owner_id: AccountId,
    market: LookupMap<AccountId, Balance>, // <TokenName: Balance> map with market's accounts and their balances
}

#[ext_contract(ext_token_holder)]
pub trait TokenHolder {
    fn new(owner_id: AccountId) -> Self;
    fn transfer(&mut self, to: AccountId, amount: Balance);
}

pub trait TokenHolder {
    fn new(owner_id: AccountId) -> Self;
    fn transfer(&mut self, to: AccountId, amount: Balance);
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        require!(!env::state_exists(), "Already initialized");
        Self {
            owner_id: owner_id.clone().into(),
            market: LookupMap::new(b"m"),
        }
    }

    #[payable]
    pub fn add_token(&mut self, token: AccountId) {
        require!(
            env::predecessor_account_id() == self.owner_id,
            "Adding token to market is allowed for owner only"
        );
        let token_balance = self.market.get(&token);
        require!(
            token_balance == None,
            "Token is already added"
        );
        require!(
            env::attached_deposit() > ADD_TOKEN_RESERVE + TOKEN_RESERVE,
            format!("Adding new token require minimum deposit {} yocto", ADD_TOKEN_RESERVE + TOKEN_RESERVE)
        );

        let deposit_to_transfer = env::attached_deposit() - ADD_TOKEN_RESERVE;
        let token_deposit = deposit_to_transfer - TOKEN_RESERVE;

        let token_id = Contract::token_id(token.clone());

        // TODO: add token to market in callback only for successfull promise result
        self.market.insert(&token, &token_deposit);

        Promise::new(token_id.clone())
            .create_account()
            .add_full_access_key(env::signer_account_pk())
            .transfer(deposit_to_transfer)
            .deploy_contract(TOKEN_CONTRACT.to_vec())
            .and(
                ext_token_holder::new(
                    env::current_account_id(),
                    token_id,
                    0, 
                    BASE_GAS
                )
            );
    }

    pub fn remove_token(&mut self, token: AccountId) {
        require!(
            env::predecessor_account_id() == self.owner_id,
            "Removing token from market is allowed for owner only"
        );
        self.market.remove(&token);

        Promise::new(Contract::token_id(token))
            .delete_account(env::signer_account_id());
    }

    #[payable]
    pub fn execute_order(&mut self, sell: AccountId, buy: AccountId, token_id: AccountId) {
        require!(sell != buy, "Can not exchange same token");
        require!(
            env::attached_deposit() > EXCHANGE_TOKEN_FEE, 
            format!("Exchanging require the deposit to be greater than {} yocto", EXCHANGE_TOKEN_FEE)
        );
        let sell_balance = self.market.get(&sell);
        let buy_balance = self.market.get(&buy);
        if let (Some(sell_balance), Some(buy_balance)) = (sell_balance, buy_balance) { 
            let dsell = env::attached_deposit() - EXCHANGE_TOKEN_FEE;
            // Instead of dY = Y - K / (X + dX) using dY = Y * dX / (X + dX), so no overflow at X * Y occurs
            // TODO: still significantly losing accuracy
            let fraction = (dsell as f64) / ((sell_balance + dsell) as f64);
            let dbuy = ((buy_balance as f64) * fraction) as Balance;
            let new_sell_balance = sell_balance + dsell;
            let new_buy_balance = buy_balance - dbuy;

            // TODO: insert in callback (or maybe insert now and revert then?)
            self.market.insert(&sell, &new_sell_balance);
            self.market.insert(&buy, &new_buy_balance);

            let buy_id = Contract::token_id(buy);
            let sell_id = Contract::token_id(sell);

            Promise::new(sell_id)
                .transfer(dsell)
                .and(
                    ext_token_holder::transfer(
                        token_id.clone(),
                        dbuy,
                        buy_id.clone(),
                        0,
                        BASE_GAS
                    )
                );
        } else {
            require!(false, "Token has not yet been added to market");
        }
    }
}

impl Contract {
    fn token_id(token: AccountId) -> AccountId {
        AccountId::new_unchecked(
            format!("{}.{}", token, env::current_account_id())
        )
    }
}

// Unit Testing
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{VMContextBuilder};
    use near_sdk::{testing_env, AccountId};

    const ONE_NEAR: Balance = 1_000_000_000_000_000_000_000_000; // 1 Ⓝ

    fn get_context(predecessor: AccountId, deposit: Balance) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor);
        builder.attached_deposit(deposit);
        builder.signer_account_pk("ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp".parse().unwrap());
        builder.current_account_id("market".parse().unwrap());
        builder
    }

    fn nikita() -> AccountId {
        "nikita".parse().unwrap()
    }

    fn denis() -> AccountId {
        "denis".parse().unwrap()
    }

    fn token1() -> AccountId {
        "token1".parse().unwrap()
    }

    fn token2() -> AccountId {
        "token2".parse().unwrap()
    }

    fn token3() -> AccountId {
        "token3".parse().unwrap()
    }

    // Tests
    #[test]
    fn test_contract_creation() {
        // arrange
        testing_env!(get_context(nikita(), 0).build());

        // act
        let contract = Contract::new(nikita());

        // assert
        assert_eq!(contract.owner_id, nikita());
    }

    #[test]
    fn test_adding_token_by_owner() {
        // arrange
        testing_env!(get_context(nikita(), ONE_NEAR).build());
        let mut contract = Contract::new(nikita());

        // act
        contract.add_token(token1());

        // assert
        let expected_token_deposit = ONE_NEAR - ADD_TOKEN_RESERVE - TOKEN_RESERVE;
        assert_eq!(contract.market.get(&token1()), Some(expected_token_deposit));
    }

    #[test]
    #[should_panic]
    fn test_adding_token_by_non_owner() {
        // arrange
        testing_env!(get_context(nikita(), 0).build());
        let mut contract = Contract::new(nikita());
        testing_env!(get_context(denis(), ONE_NEAR).build());

        // act
        contract.add_token(token1());

        // assert
        assert!(false);
    }

    #[test]
    fn test_removing_token_by_owner() {
        // arrange
        testing_env!(get_context(nikita(), ONE_NEAR).build());
        let mut contract = Contract::new(nikita());
        contract.add_token(token1());

        // act
        contract.remove_token(token1());

        // assert
        assert_eq!(contract.market.get(&token1()), None);
    }

    #[test]
    #[should_panic]
    fn test_removing_token_by_non_owner() {
        // arrange
        testing_env!(get_context(nikita(), ONE_NEAR).build());
        let mut contract = Contract::new(nikita());
        contract.add_token(token1());
        testing_env!(get_context(denis(), 0).build());

        // act
        contract.remove_token(token1());

        // assert
        assert!(false);
    }

    #[test]
    #[should_panic]
    fn test_exchange_non_existing_tokens() {
        // arrange
        testing_env!(get_context(nikita(), 100).build());
        let mut contract = Contract::new(nikita());

        // act & assert
        contract.execute_order(token1(), token1(), denis());
    }

    #[test]
    #[should_panic]
    fn test_exchange_with_low_deposit() {
        // arrange
        testing_env!(get_context(nikita(), ONE_NEAR).build());
        let mut contract = Contract::new(nikita());
        contract.add_token(token1());
        contract.add_token(token2());
        testing_env!(get_context(denis(), EXCHANGE_TOKEN_FEE).build());

        // act & assert
        contract.execute_order(token1(), token2(), denis());
    }

    #[test]
    #[should_panic]
    fn test_exchange_same_token() {
        // arrange
        testing_env!(get_context(nikita(), ONE_NEAR).build());
        let mut contract = Contract::new(nikita());
        contract.add_token(token1());

        // act & assert
        contract.execute_order(token1(), token1(), denis());
    }

    #[test]
    fn test_exchange_tokens() {
        // arrange
        let near_100_with_add_fee = ONE_NEAR * 100 + ADD_TOKEN_RESERVE + TOKEN_RESERVE;
        let near_25_with_exchange_fee = ONE_NEAR * 25 + EXCHANGE_TOKEN_FEE;

        testing_env!(get_context(nikita(), near_100_with_add_fee).build());
        let mut contract = Contract::new(nikita());
        contract.add_token(token1());

        testing_env!(get_context(nikita(), near_100_with_add_fee).build());
        contract.add_token(token2());

        testing_env!(get_context(nikita(), near_100_with_add_fee).build());
        contract.add_token(token3());
        
        testing_env!(get_context(denis(), near_25_with_exchange_fee).build());

        // act

        // t1 = 100
        // t2 = 100
        // k = 10_000
        // dt1 = 25
        // dt2 = 100 - 10_000 / (100 + 25) = 100 - 10_000 / 125 = 100 - 80 == 20
        // newT1 = 125
        // newT2 = 79 or 80  due to float point errors
        contract.execute_order(token1(), token2(), denis());

        // t1 = 125
        // t3 = 100
        // k = 12_500
        // dt1 = 25
        // dt3 = 100 - 12_500 / (125 + 25) = 100 - 12_500 / 150 = 100 - 83 == 17
        // newT1 = 150
        // newT3 = 83 or 84
        contract.execute_order(token1(), token3(), denis());

        // assert
        assert_eq!(contract.market.get(&token1()).unwrap() / ONE_NEAR, 150);
        assert_eq!(contract.market.get(&token2()).unwrap() / ONE_NEAR, 79);
        assert_eq!(contract.market.get(&token3()).unwrap() / ONE_NEAR, 83);
    }

    #[test]
    fn test_equilibrium_calculation_overflow() {
        // arrange
        testing_env!(get_context(nikita(), ONE_NEAR * 100 + ADD_TOKEN_RESERVE + TOKEN_RESERVE).build());
        let mut contract = Contract::new(nikita());
        contract.add_token(token1());
        contract.add_token(token2());
        
        testing_env!(get_context(denis(), 25_000_000_000_000_000_000_000_000).build());

        // act
        contract.execute_order(token1(), token2(), denis());

        // assert
        assert_eq!(contract.market.get(&token1()), Some(124_990_000_000_000_000_000_000_000));
        assert_eq!(contract.market.get(&token2()), Some(80_006_400_512_040_962_227_175_424));
    }
}
