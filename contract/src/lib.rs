use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env, log, near_bindgen,
    serde::{Deserialize, Serialize},
    AccountId,
};

//TODO: 追加機能集
//TODO: ユーザ1人一つしか使用できないようにする
//TODO: アカウント所持者はバイクの数を増やせる
//TODO: 使用しているユーザにしかリターンボタンを見せないようにする

// Define the default message
const DEFAULT_MESSAGE: &str = "Hello";
//TODO: initで指定した数のvecにしたい
const NUMBER_OF_BIKES: usize = 5;

const USE_AMOUNT: u128 = 4_000_000_000_000_000_000_000_000;
const REPAIR_AMOUNT: u128 = 2_000_000_000_000_000_000_000_000;

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

    fn use_bike(&mut self) {
        assert!(self.available());
        *self = Bike::InUse(env::predecessor_account_id());
    }

    fn inspect_bike(&mut self) {
        assert!(self.available());
        *self = Bike::Cleaning(env::predecessor_account_id());
    }

    fn return_bike(&mut self) {
        assert!(
            *self == Bike::InUse(env::predecessor_account_id())
                || *self == Bike::Cleaning(env::predecessor_account_id())
        );
        *self = Bike::Available;
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

    pub fn get_bikes(&self) -> Vec<bool> {
        self.bikes.iter().map(|bike| bike.available()).collect()
    }

    pub fn use_bike(&mut self, index: usize) {
        self.bikes[index].use_bike();
    }

    pub fn inspect_bike(&mut self, index: usize) {
        self.bikes[index].inspect_bike();
    }

    pub fn return_bike(&mut self, index: usize) {
        self.bikes[index].return_bike();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_default_greeting() {
        let contract = Contract::default();
        assert_eq!(contract.get_greeting(), "Hello".to_string());
        for bike in contract.bikes {
            assert_eq!(bike, Bike::Available);
        }
    }

    #[test]
    fn set_then_get_greeting() {
        let mut contract = Contract::default();
        contract.set_greeting("howdy".to_string());
        assert_eq!(contract.get_greeting(), "howdy".to_string());
    }

    //TODO: integration_test動かす
    #[test]
    fn use_return_inspect() {
        let mut contract = Contract::default();
        let test_number = 1;
        assert_eq!(contract.bikes[test_number], Bike::Available);
        contract.use_bike(test_number);
        assert_eq!(
            contract.bikes[test_number],
            Bike::InUse(env::predecessor_account_id())
        );
        //TODO: アサーションテスト
        //contract.use_bike(test_number);
        //assert_eq!(
        //    contract.bikes[test_number],
        //    Bike::InUse(env::predecessor_account_id())
        //);
        contract.return_bike(test_number);
        assert_eq!(contract.bikes[test_number], Bike::Available);
        //    contract.return_bike(test_number);
        //    assert_eq!(contract.bikes[test_number], Bike::Available);
        contract.inspect_bike(test_number);
        assert_eq!(
            contract.bikes[test_number],
            Bike::Cleaning(env::predecessor_account_id())
        );
        //    contract.use_bike(test_number);
        //    assert_eq!(
        //        contract.bikes[test_number],
        //        Bike::Cleaning(env::predecessor_account_id())
        //    );
        contract.return_bike(test_number);
        assert_eq!(contract.bikes[test_number], Bike::Available);
    }
}
