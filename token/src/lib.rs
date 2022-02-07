use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{
    env, near_bindgen, require,
    AccountId, Promise, Balance, PanicOnDefault
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

    pub fn transfer(&mut self, to: AccountId, amount: Balance) {
        require!(
            env::predecessor_account_id() == self.owner_id,
            "Transfering token is allowed for owner only"
        );
        Promise::new(to).transfer(amount);
    }
}


// Unit Testing
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{VMContextBuilder};
    use near_sdk::{testing_env, AccountId};

    fn get_context(predecessor: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor);
        builder
    }

    fn nikita() -> AccountId {
        "nikita".parse().unwrap()
    }

    fn denis() -> AccountId {
        "denis".parse().unwrap()
    }

    // Tests
    #[test]
    fn test_contract_creation() {
        // arrange
        testing_env!(get_context(nikita()).build());

        // act
        let contract = TokenHolderContract::new(nikita());

        // assert
        assert_eq!(contract.owner_id, nikita());
    }

    #[test]
    #[should_panic]
    fn test_transfer_non_owner() {
        // arrange
        testing_env!(get_context(nikita()).build());
        let mut contract = TokenHolderContract::new(nikita());
        
        // act
        testing_env!(get_context(denis()).build());
        contract.transfer(nikita(), 100);

        // assert
        assert_eq!(contract.owner_id, nikita());
    }

    #[test]
    fn test_transfer() {
        // arrange
        testing_env!(get_context(nikita()).build());
        let mut contract = TokenHolderContract::new(nikita());
        
        // act & assert
        contract.transfer(nikita(), 100);
    }
}
