/*
 * Example smart contract written in RUST
 *
 * Learn more about writing NEAR smart contracts with Rust:
 * https://near-docs.io/develop/Contract
 *
 */

// TODO: プロジェクトの名前を変える

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    log, near_bindgen,
    serde::{Deserialize, Serialize},
    AccountId,
};

// Define the default message
const DEFAULT_MESSAGE: &str = "Hello";
// バイクの数
const NUMBER_OF_BIKES: usize = 5;

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum BikeState {
    Available,
    InUse,
    Cleaning,
}

impl Default for BikeState {
    fn default() -> Self {
        BikeState::Available
    }
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Bike {
    account_id: AccountId,
    state: BikeState,
}

impl Default for Bike {
    fn default() -> Self {
        Self {
            account_id: "alice.near".parse().unwrap(), //TODO: tmp
            state: BikeState::default(),
        }
    }
}

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    message: String,
    bikes: Vec<Bike>,
}

// Define the default, which automatically initializes the contract
impl Default for Contract {
    fn default() -> Self {
        Self {
            message: DEFAULT_MESSAGE.to_string(),
            bikes: {
                let mut v = Vec::new();
                let mut index = 0;
                while index < NUMBER_OF_BIKES {
                    v.push(Bike::default());
                    index += 1;
                }
                v
            },
            //TODO: initで指定した数のvecにしたい
        }
    }
}

// TODO: account_id利用
// TODO: 送金する

// Implement the contract structure
#[near_bindgen]
impl Contract {
    // Public method - returns the greeting saved, defaulting to DEFAULT_MESSAGE
    pub fn get_greeting(&self) -> String {
        return self.message.clone();
    }

    // Public method - accepts a greeting, such as "howdy", and records it
    pub fn set_greeting(&mut self, message: String) {
        // Use env::log to record logs permanently to the blockchain!
        log!("Saving greeting {}", message);
        self.message = message;
    }

    // TODO: アカウントに合わせて整形した(各ブール値もつけたものを返す)
    pub fn get_bikes(&self) -> &[Bike] {
        &self.bikes
    }

    // 使用可能かどうか
    fn available(&self, number: usize) -> bool {
        self.bikes[number].state == BikeState::Available
    }

    // 使用中かどうか
    fn in_use(&self, number: usize) -> bool {
        self.bikes[number].state == BikeState::InUse
    }

    // 清掃中かどうか
    fn cleaning(&self, number: usize) -> bool {
        self.bikes[number].state == BikeState::Cleaning
    }

    pub fn use_bike(&mut self, number: usize) {
        if self.available(number) {
            self.bikes[number].state = BikeState::InUse;
        }
    }

    pub fn return_bike(&mut self, number: usize) {
        if self.in_use(number) || self.cleaning(number) {
            self.bikes[number].state = BikeState::Available;
        }
    }

    pub fn clean_bike(&mut self, number: usize) {
        if self.available(number) {
            self.bikes[number].state = BikeState::Cleaning;
        }
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_default_greeting() {
        let contract = Contract::default();
        // this test did not call set_greeting so should return the default "Hello" greeting
        assert_eq!(contract.get_greeting(), "Hello".to_string());
        //TODO: テストもっとちゃんと書く
        assert_eq!(contract.bikes[1].state, BikeState::Available);
    }

    #[test]
    fn set_then_get_greeting() {
        let mut contract = Contract::default();
        contract.set_greeting("howdy".to_string());
        assert_eq!(contract.get_greeting(), "howdy".to_string());
    }

    //TODO: テストもっとちゃんと書く
    #[test]
    fn use_return_clean() {
        let mut contract = Contract::default();
        let test_number = 1;
        assert_eq!(contract.bikes[test_number].state, BikeState::Available);
        contract.use_bike(test_number);
        assert_eq!(contract.bikes[test_number].state, BikeState::InUse);
        contract.use_bike(test_number);
        assert_eq!(contract.bikes[test_number].state, BikeState::InUse);
        contract.return_bike(test_number);
        assert_eq!(contract.bikes[test_number].state, BikeState::Available);
        contract.return_bike(test_number);
        assert_eq!(contract.bikes[test_number].state, BikeState::Available);
        contract.clean_bike(test_number);
        assert_eq!(contract.bikes[test_number].state, BikeState::Cleaning);
        contract.use_bike(test_number);
        assert_eq!(contract.bikes[test_number].state, BikeState::Cleaning);
        contract.return_bike(test_number);
        assert_eq!(contract.bikes[test_number].state, BikeState::Available);
    }
}
