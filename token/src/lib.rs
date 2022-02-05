use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::{
    env,
    near_bindgen, ext_contract, require,
    AccountId, PanicOnDefault, Promise, Gas, PromiseResult, Balance
};

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct TokenHolderContract {
    owner_id: AccountId,
}

#[near_bindgen]
impl TokenHolderContract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        require!(!env::state_exists(), "Already initialized");
        Self {
            owner_id: owner_id.clone().into(),
        }
    }

    #[payable]
    pub fn transfer(&mut self, to: AccountId) {

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

    // Tests
    #[test]
    fn test_contract_creation() {

    }
}
