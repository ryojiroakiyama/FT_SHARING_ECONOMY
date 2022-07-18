use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    json_types::U128,
    log, near_bindgen,
    serde::{Deserialize, Serialize},
    serde_json, AccountId, PromiseResult,
};
use std::convert::TryInto;

//TODO: 追加機能集
//TODO: ユーザ1人一つしか使用できないようにする
//TODO: アカウント所持者はバイクの数を増やせる
//TODO: 使用しているユーザにしかリターンボタンを見せないようにする

// TODO: ユーザ間送金機能つける

// Define the default message
const DEFAULT_MESSAGE: &str = "Hello";

const NUMBER_OF_BIKES: usize = 5;

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

    // TODO: 指定することでメソッドを使える型を限定できるかも
    fn be_in_use(&mut self) {
        assert!(self.available(), "Not available");
        *self = Bike::InUse(env::predecessor_account_id());
    }

    fn be_inspected(&mut self) {
        assert!(self.available(), "Not available");
        *self = Bike::Inspection(env::predecessor_account_id());
    }

    //TODO: match文でエラー内容分ける
    fn be_returned(&mut self) {
        assert!(
            *self == Bike::InUse(env::predecessor_account_id())
                || *self == Bike::Inspection(env::predecessor_account_id()),
            "Not in use or inspection"
        );
        *self = Bike::Available;
    }
}

// TODO: contractの名前変える
// TODO: accountIDを持って, 特手のアカウントからの処理ができるようにする
// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    message: String,
    bikes: Vec<Bike>,
}

// TODO: initで指定した数のバイクを作る構成にする
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
    pub fn cross_method(&mut self) {
        let contract_accountid = "my_ft.testnet".to_string();
        // Create a new promise, which will create a new (empty) ActionReceipt
        let promise_id = env::promise_batch_create(
            &contract_accountid.try_into().unwrap(), // the recipient of this ActionReceipt (contract account id)
        );

        // attach a function call action to the ActionReceipt
        env::promise_batch_action_function_call(
            promise_id, // associate the function call with the above Receipt via promise_id
            &"ft_transfer".to_string(), // the function call will invoke the ft_balance_of method on the wrap.testnet
            &serde_json::json!({ "account_id": "my_ft.testnet".to_string() }) // method arguments
                .to_string()
                .into_bytes(),
            0,                                // amount of yoctoNEAR to attach
            near_sdk::Gas(5_000_000_000_000), // gas to attach
        );

        // Create another promise, which will create another (empty) ActionReceipt.
        // This time, the ActionReceipt is dependent on the previous receipt
        let callback_promise_id = env::promise_batch_then(
            promise_id, // postpone until a DataReceipt associated with promise_id is received
            &env::current_account_id(), // the recipient of this ActionReceipt (&self)
        );

        // attach a function call action to the ActionReceipt
        env::promise_batch_action_function_call(
            callback_promise_id, // associate the function call with callback_promise_id
            &"my_callback".to_string(), // the function call will be a callback function
            b"{}",               // method arguments
            0,                   // amount of yoctoNEAR to attach
            near_sdk::Gas(5_000_000_000_000), // gas to attach
        );

        // return the resulting DataReceipt from callback_promise_id as the result of this function
        env::promise_return(callback_promise_id);
    }

    pub fn my_callback(&self) -> String {
        assert_eq!(env::promise_results_count(), 1, "This is a callback method");

        // handle the result from the cross contract call this method is a callback for
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => "oops!".to_string(),
            PromiseResult::Successful(result) => {
                let balance = near_sdk::serde_json::from_slice::<U128>(&result).unwrap();
                if balance.0 > 100000 {
                    log!("=============={}", balance.0);
                    "Wow!".to_string()
                } else {
                    log!("-------------->{}", balance.0);
                    "Hmmmm".to_string()
                }
            }
        }
    }

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

    //TODO: accountIdのstringも受け取って, returnなどについても用意する
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

    #[test]
    fn cross() {
        let contract = Contract::default();
        contract.cross_method();
    }
}
