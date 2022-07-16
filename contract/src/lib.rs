/*
 * Example smart contract written in RUST
 *
 * Learn more about writing NEAR smart contracts with Rust:
 * https://near-docs.io/develop/Contract
 *
 */

const USE_AMOUNT: u128 = 4_000_000_000_000_000_000_000_000;
const REPAIR_AMOUNT: u128 = 2_000_000_000_000_000_000_000_000;

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env, log, near_bindgen,
    serde::{Deserialize, Serialize},
    AccountId,
};

// Define the default message
const DEFAULT_MESSAGE: &str = "Hello";
// バイクの数
const NUMBER_OF_BIKES: usize = 5;

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum Bike {
    Available,
    InUse(AccountId),
    Cleaning(AccountId),
}

impl Default for Bike {
    fn default() -> Self {
        Bike::Available
    }
}

impl Bike {
    // 使用可能かどうか
    fn available(&self) -> bool {
        *self == Bike::Available
    }

    // 使用中かどうか
    fn using(&self) -> bool {
        *self == Bike::InUse(env::current_account_id())
    }

    // 清掃中かどうか
    fn inspecting(&self) -> bool {
        *self == Bike::Cleaning(env::current_account_id())
    }
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct BikeForFrontEnd {
    available: bool,
    using: bool,
    inspecting: bool,
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

    // TODO: iter使ったもっと楽な書き方あるはず
    pub fn get_bikes(&self) -> Vec<BikeForFrontEnd> {
        let mut v = Vec::new();
        let mut index = 0;
        while index < self.bikes.len() {
            v.push(BikeForFrontEnd {
                available: self.bikes[index].available(),
                using: self.bikes[index].using(),
                inspecting: self.bikes[index].inspecting(),
            });
            index += 1;
        }
        v
    }
    //TODO: ユーザ1人一つしか使用できないようにする機能追加を推薦してもいいかも
    //TODO: フロント側でボタンの押し足は決める？

    pub fn use_bike(&mut self, index: usize) {
        assert!(self.bikes[index].available());
        self.bikes[index] = Bike::InUse(env::current_account_id());
    }

    pub fn return_bike(&mut self, index: usize) {
        assert!(self.bikes[index].using() || self.bikes[index].inspecting());
        self.bikes[index] = Bike::Available;
    }

    pub fn inspect_bike(&mut self, index: usize) {
        assert!(self.bikes[index].available());
        self.bikes[index] = Bike::Cleaning(env::current_account_id());
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
        assert_eq!(contract.bikes[1], Bike::Available);
    }

    #[test]
    fn set_then_get_greeting() {
        let mut contract = Contract::default();
        contract.set_greeting("howdy".to_string());
        assert_eq!(contract.get_greeting(), "howdy".to_string());
    }

    //TODO: テストもっとちゃんと書く
    #[test]
    fn use_return_inspect() {
        let mut contract = Contract::default();
        let test_number = 1;
        assert_eq!(contract.bikes[test_number], Bike::Available);
        contract.use_bike(test_number);
        assert_eq!(
            contract.bikes[test_number],
            Bike::InUse(env::current_account_id())
        );
        contract.use_bike(test_number);
        assert_eq!(
            contract.bikes[test_number],
            Bike::InUse(env::current_account_id())
        );
        contract.return_bike(test_number);
        assert_eq!(contract.bikes[test_number], Bike::Available);
        contract.return_bike(test_number);
        assert_eq!(contract.bikes[test_number], Bike::Available);
        contract.inspect_bike(test_number);
        assert_eq!(
            contract.bikes[test_number],
            Bike::Cleaning(env::current_account_id())
        );
        contract.use_bike(test_number);
        assert_eq!(
            contract.bikes[test_number],
            Bike::Cleaning(env::current_account_id())
        );
        contract.return_bike(test_number);
        assert_eq!(contract.bikes[test_number], Bike::Available);
    }
}
