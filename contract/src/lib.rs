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

// TODO: 送金する

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
    Inspection(AccountId),
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

    fn be_in_use(&mut self) {
        assert!(self.available(), "Not available");
        *self = Bike::InUse(env::predecessor_account_id());
    }

    fn be_inspected(&mut self) {
        assert!(self.available(), "Not available");
        *self = Bike::Inspection(env::predecessor_account_id());
    }

    fn be_returned(&mut self) {
        assert!(
            *self == Bike::InUse(env::predecessor_account_id())
                || *self == Bike::Inspection(env::predecessor_account_id()),
            "Not in use or inspection"
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

    pub fn get_bike_states(&self) -> Vec<bool> {
        self.bikes.iter().map(|bike| bike.available()).collect()
    }

    pub fn use_bike(&mut self, index: usize) {
        self.bikes[index].be_in_use();
    }

    pub fn inspect_bike(&mut self, index: usize) {
        self.bikes[index].be_inspected();
    }

    pub fn return_bike(&mut self, index: usize) {
        self.bikes[index].be_returned();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_default() {
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

    #[test]
    fn contract_use_bike_check_status() {
        let mut contract = Contract {
            message: String::from("a"),
            bikes: vec![Bike::Available, Bike::Available, Bike::Available],
        };
        let index = 1;

        // 初期状態をチェック
        for s in contract.get_bike_states() {
            assert!(s);
        }

        // バイクを使用, 状態をチェック
        contract.use_bike(index);
        assert!(!contract.bikes[index].available());

        // バイクを返却, 状態をチェック
        contract.return_bike(index);
        for s in contract.get_bike_states() {
            assert!(s);
        }
    }

    #[test]
    fn use_inspect_return_bike() {
        // 初期状態チェック
        let mut bike = Bike::default();
        assert!(bike.available());

        // バイク使用, 状態チェック
        bike.be_in_use();
        assert_eq!(bike, Bike::InUse(env::predecessor_account_id()));

        // バイク返却, 状態チェック
        bike.be_returned();
        assert_eq!(bike, Bike::Available);

        // バイク点検, 状態チェック
        bike.be_inspected();
        assert_eq!(bike, Bike::Inspection(env::predecessor_account_id()));

        // バイク返却, 状態チェック
        bike.be_returned();
        assert_eq!(bike, Bike::Available);
    }

    #[test]
    #[should_panic(expected = "Not available")]
    fn duplicate_use() {
        let mut bike = Bike::InUse(env::predecessor_account_id());
        bike.be_in_use();
    }

    #[test]
    #[should_panic(expected = "Not available")]
    fn duplicate_inspect() {
        let mut bike = Bike::InUse(env::predecessor_account_id());
        bike.be_inspected();
    }

    #[test]
    #[should_panic(expected = "Not in use or inspection")]
    fn duplicate_return() {
        let mut bike = Bike::Available;
        bike.be_returned();
    }
}
