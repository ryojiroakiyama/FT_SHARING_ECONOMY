/*
 * Example smart contract written in RUST
 *
 * Learn more about writing NEAR smart contracts with Rust:
 * https://near-docs.io/develop/Contract
 *
 */

// TODO: プロジェクトの名前を変える

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{log, near_bindgen};

// Define the default message
const DEFAULT_MESSAGE: &str = "Hello";
// バイクの数
const NUMBER_OF_BIKES: usize = 5;

#[near_bindgen]
#[derive(Clone, Copy, PartialEq, Debug, BorshDeserialize, BorshSerialize)]
enum State {
    Available,
    InUse,
    Cleaning,
}

impl Default for State {
    fn default() -> Self {
        State::Available
    }
}

#[near_bindgen]
impl State {
    // 使用可能かどうか
    fn available(&self) -> bool {
        *self == State::Available
    }

    // 使用中かどうか
    fn in_use(&self) -> bool {
        *self == State::InUse
    }

    // 清掃中かどうか
    fn cleaning(&self) -> bool {
        *self == State::Cleaning
    }

    // バイクを使用
    fn use_bike(&mut self) {
        if self.available() {
            *self = State::InUse;
        }
    }

    // バイクを返却
    fn return_bike(&mut self) {
        if self.in_use() || self.cleaning() {
            *self = State::Available;
        }
    }

    // バイクを清掃
    fn clean_bike(&mut self) {
        if self.available() {
            *self = State::Cleaning;
        }
    }
}

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    message: String,
    bikes: [State; NUMBER_OF_BIKES],
}

// Define the default, which automatically initializes the contract
impl Default for Contract {
    fn default() -> Self {
        Self {
            message: DEFAULT_MESSAGE.to_string(),
            bikes: [State::Available; NUMBER_OF_BIKES],
        }
    }
}

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

    // TODO: 直接enum側が呼べるならそっちの方がいい
    pub fn use_bike(&mut self, number: usize) {
        self.bikes[number].use_bike()
    }

    pub fn return_bike(&mut self, number: usize) {
        self.bikes[number].return_bike()
    }

    pub fn clean_bike(&mut self, number: usize) {
        self.bikes[number].clean_bike()
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
        assert_eq!(contract.bikes[1], State::Available);
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
        assert_eq!(contract.bikes[test_number], State::Available);
        contract.bikes[test_number].use_bike();
        assert_eq!(contract.bikes[test_number], State::InUse);
        contract.bikes[test_number].use_bike();
        assert_eq!(contract.bikes[test_number], State::InUse);
        contract.bikes[test_number].return_bike();
        assert_eq!(contract.bikes[test_number], State::Available);
        contract.bikes[test_number].return_bike();
        assert_eq!(contract.bikes[test_number], State::Available);
        contract.bikes[test_number].clean_bike();
        assert_eq!(contract.bikes[test_number], State::Cleaning);
        contract.bikes[test_number].use_bike();
        assert_eq!(contract.bikes[test_number], State::Cleaning);
        contract.bikes[test_number].return_bike();
        assert_eq!(contract.bikes[test_number], State::Available);
    }
}
