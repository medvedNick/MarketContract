use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::{
    env,
    near_bindgen, ext_contract, require,
    AccountId, PanicOnDefault, Promise, Gas, PromiseResult, Balance
};

const TOKEN_CONTRACT: &[u8] = include_bytes!("../../token/res/token_contract.wasm");
const GAS_FOR_ACCOUNT_CALLBACK: Gas = Gas(110_000_000_000_000);

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Contract {
    owner_id: AccountId,
    market: LookupMap<AccountId, Balance>,
}

#[ext_contract(ext_self)]
pub trait AfterAccountCreate {
    fn callback_after_create_account(
        &mut self,
    ) -> bool;
}

pub trait AfterAccountCreate {
    fn callback_after_create_account(
        &mut self,
    ) -> bool;
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

        let initial_balance = env::attached_deposit();

        // let token_id = AccountId::new_unchecked(
        //     format!("{}.{}", token, env::current_account_id())
        // );
        // Promise::new(token_id.clone())
        //     .create_account()
        //     .add_full_access_key(env::signer_account_pk())
        //     .transfer(initial_balance)
        //     .then(
        //         ext_self::callback_after_create_account(
        //             token_id.clone(),
        //             0,
        //             GAS_FOR_ACCOUNT_CALLBACK,
        //         )
        //     );
        self.market.insert(&token, &initial_balance);
    }

    #[payable]
    pub fn execute_order(&mut self, sell: AccountId, buy: AccountId) {
        let sell_balance = self.market.get(&sell);
        let buy_balance = self.market.get(&buy);
        if let (Some(sell_balance), Some(buy_balance)) = (sell_balance, buy_balance) { 
            let dsell = env::attached_deposit();
            // TODO: still significantly losing accuracy
            let fraction = (dsell as f64) / (sell_balance as f64 + dsell as f64);
            let dbuy = ((buy_balance as f64) * fraction) as Balance;
            let new_sell_balance = sell_balance + dsell;
            let new_buy_balance = buy_balance - dbuy;
            self.market.insert(&sell, &new_sell_balance);
            self.market.insert(&buy, &new_buy_balance);
            Promise::new(env::predecessor_account_id())
                .transfer(dbuy);
        } else {
            require!(false, "Token has not yet been added to market");
        }
    }
}

#[near_bindgen]
impl AfterAccountCreate for Contract {
    #[private]
    fn callback_after_create_account(
        &mut self,
    ) -> bool {
        require!(
            env::promise_results_count() == 1, 
            "Expected 1 promise result."
        );
        match env::promise_result(0) {
            PromiseResult::NotReady => {
                unreachable!()
            }
            PromiseResult::Successful(_) => {
                env::log_str("Account created");
                true
                // let creation_succeeded: bool = serde_json::from_slice(&creation_result)
                //     .expect("Could not turn result from account creation into boolean.");
                // if creation_succeeded {
                //     true
                // } else {
                //     false
                // }
            }
            PromiseResult::Failed => {
                false
            }
        }
    }
}


// Unit Testing
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{VMContextBuilder};
    use near_sdk::{testing_env, AccountId};

    fn get_context(predecessor: AccountId, deposit: Balance) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor);
        builder.attached_deposit(deposit);
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
        testing_env!(get_context(nikita(), 0).build());

        let contract = Contract::new(nikita());

        assert_eq!(contract.owner_id, nikita());
    }

    #[test]
    fn test_adding_token_by_owner() {
        // arrange
        testing_env!(get_context(nikita(), 100).build());
        let owner = nikita();
        let mut contract = Contract::new(owner);
        let token1 = token1();

        // act
        contract.add_token(token1.clone());

        // assert
        assert_eq!(contract.market.get(&token1), Some(100));
    }

    #[test]
    #[should_panic]
    fn test_adding_token_by_non_owner() {
        // arrange
        testing_env!(get_context(nikita(), 100).build());
        let owner = nikita();
        let mut contract = Contract::new(owner);
        let token1 = token1();
        testing_env!(get_context(denis(), 100).build());

        // act
        contract.add_token(token1.clone());

        // assert
        assert!(false);
    }

    #[test]
    #[should_panic]
    fn test_exchange_non_existing_tokens() {
        // arrange
        testing_env!(get_context(nikita(), 100).build());
        let owner = nikita();
        let mut contract = Contract::new(owner);
        let token1 = token1();

        // act
        contract.execute_order(token1.clone(), token1);

        // assert
        assert!(false);
    }

    #[test]
    fn test_exchange_tokens() {
        // arrange
        testing_env!(get_context(nikita(), 100).build());
        let owner = nikita();
        let mut contract = Contract::new(owner);
        let token1 = token1();
        let token2 = token2();
        let token3 = token3();
        contract.add_token(token1.clone());
        contract.add_token(token2.clone());
        contract.add_token(token3.clone());
        
        testing_env!(get_context(denis(), 25).build());

        // act

        // t1 = 100
        // t2 = 100
        // k = 10_000
        // dt1 = 25
        // dt2 = 100 - 10_000 / (100 + 25) = 100 - 10_000 / 125 = 100 - 80 == 20
        // newT1 = 125
        // newT2 = 80
        contract.execute_order(token1.clone(), token2.clone());

        // t1 = 125
        // t3 = 100
        // k = 12_500
        // dt1 = 25
        // dt3 = 100 - 12_500 / (125 + 25) = 100 - 12_500 / 150 = 100 - 83 == 17
        // newT1 = 150
        // newT3 = 83 or 84 due to float point errors
        contract.execute_order(token1.clone(), token3.clone());

        // assert
        assert_eq!(contract.market.get(&token1), Some(150));
        assert_eq!(contract.market.get(&token2), Some(80));
        assert_eq!(contract.market.get(&token3), Some(84));
    }

    #[test]
    fn test_equilibrium_calculation_overflow() {
        // arrange
        testing_env!(get_context(nikita(), 100_000_000_000_000_000_000_000_000).build());
        let owner = nikita();
        let mut contract = Contract::new(owner);
        let token1 = token1();
        let token2 = token2();
        contract.add_token(token1.clone());
        contract.add_token(token2.clone());
        
        testing_env!(get_context(denis(), 25_000_000_000_000_000_000_000_000).build());

        // act
        contract.execute_order(token1.clone(), token2.clone());

        // assert
        assert_eq!(contract.market.get(&token1), Some(125_000_000_000_000_000_000_000_000));
        assert_eq!(contract.market.get(&token2), Some(79_999_999_999_999_998_188_060_672));
    }
}
