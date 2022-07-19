use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env, log, near_bindgen,
    serde::{Deserialize, Serialize},
    AccountId,
};

//TODO: 追加機能集
//TODO: ユーザ1人一つしか使用できないようにする
//TODO: アカウント所持者はバイクの数を増やせる

// TODO: ユーザ間送金機能つける

// Define the default message
const DEFAULT_MESSAGE: &str = "Hello";
const NUMBER_OF_BIKES: usize = 5;

// Bikeの状態
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum Bike {
    Available,             // 使用可能
    InUse(AccountId),      // AccountIdによって使用中
    Inspection(AccountId), // AccountIdによって点検中
}

// デフォルトでは使用可能状態
impl Default for Bike {
    fn default() -> Self {
        Bike::Available
    }
}

// Bikeに関する機能をメソッドでまとめる
impl Bike {
    // 使用可能かどうかチェック
    fn is_available(&self) -> bool {
        *self == Bike::Available
    }

    // 指定アカウントによって使用中かどうかチェック
    fn is_in_use_by(&self, account_id: AccountId) -> bool {
        *self == Bike::InUse(account_id)
    }

    // 指定アカウントによって点検中かどうかチェック
    fn is_inspected_by(&self, account_id: AccountId) -> bool {
        *self == Bike::Inspection(account_id)
    }

    // 使用中に状態を変更,
    // このバイクを使用中 or 点検中のアカウントのみ変更可能
    fn be_available(&mut self) {
        //predecessor_account_id(): コントラクトを呼び出しているアカウント
        assert!(
            self.is_in_use_by(env::predecessor_account_id())
                || self.is_inspected_by(env::predecessor_account_id()),
            "Not in use or inspection"
        );
        *self = Bike::Available;
    }

    // 呼び出しアカウントによって使用中に状態を変更
    fn be_in_use(&mut self) {
        assert!(self.is_available(), "Not available");
        *self = Bike::InUse(env::predecessor_account_id());
    }

    // 呼び出しアカウントによって点検中に状態を変更
    fn be_inspection(&mut self) {
        assert!(self.is_available(), "Not available");
        *self = Bike::Inspection(env::predecessor_account_id());
    }
}

//#[near_bindgen]
//#[derive(BorshDeserialize, BorshSerialize)]
//pub struct BikeInfoForOneUser {
//    available: bool,
//    using: bool,
//    inspecting: bool,
//}

// コントラクトの定義
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    message: String,
    bikes: Vec<Bike>,
}

//TODO: initに変更して, 指定した数のsizeでvector作る, DefaultOnPanicにする
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

    //TODO: accountIdのstringも受け取って, returnなどについても用意する
    pub fn get_bike_states(&self) -> Vec<bool> {
        self.bikes.iter().map(|bike| bike.is_available()).collect()
    }

    // 指定バイクを返却
    pub fn return_bike(&mut self, index: usize) {
        self.bikes[index].be_available();
    }

    // 指定バイクを使用
    pub fn use_bike(&mut self, index: usize) {
        self.bikes[index].be_in_use();
    }

    // 指定バイクを点検
    pub fn inspect_bike(&mut self, index: usize) {
        self.bikes[index].be_inspection();
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

    // バイクの状態を変更して, get_bike_statesの結果を確認
    #[test]
    fn change_state_then_get_states() {
        let mut contract = Contract::default();
        let index = 1;

        // 初期状態をチェック
        for s in contract.get_bike_states() {
            assert!(s);
        }

        // バイクを使用, 状態をチェック
        contract.use_bike(index);
        assert!(!contract.bikes[index].is_available());

        // バイクを返却, 状態をチェック
        contract.return_bike(index);
        for s in contract.get_bike_states() {
            assert!(s);
        }
    }

    // バイクを使用, 点検, 返却, 状態チェック
    #[test]
    fn use_inspect_return_bike() {
        // 初期状態チェック
        let mut bike = Bike::default();
        assert!(bike.is_available());

        // バイク使用, 状態チェック
        bike.be_in_use();
        assert!(bike.is_in_use_by(env::predecessor_account_id()));

        // バイク返却, 状態チェック
        bike.be_available();
        assert!(bike.is_available());

        // バイク点検, 状態チェック
        bike.be_inspection();
        assert!(bike.is_inspected_by(env::predecessor_account_id()));

        // バイク返却, 状態チェック
        bike.be_available();
        assert!(bike.is_available());
    }

    // 重複してバイクを使用->パニックを起こすか確認
    #[test]
    #[should_panic(expected = "Not available")]
    fn duplicate_use() {
        let mut bike = Bike::InUse(env::predecessor_account_id());
        bike.be_in_use();
    }

    // 重複してバイクを点検->パニックを起こすか確認
    #[test]
    #[should_panic(expected = "Not available")]
    fn duplicate_inspect() {
        let mut bike = Bike::Inspection(env::predecessor_account_id());
        bike.be_inspection();
    }

    // 重複してバイクを使用可能に->パニックを起こすか確認
    #[test]
    #[should_panic(expected = "Not in use or inspection")]
    fn duplicate_return() {
        let mut bike = Bike::Available;
        bike.be_available();
    }

    // 別のアカウントが使用中に使用可能に変更->パニックを起こすか確認
    #[test]
    #[should_panic(expected = "Not in use or inspection")]
    fn return_by_other_account() {
        // env::predecessor_account_id()に接頭語"a"をつけて, 別のアカウントを表す文字列作成
        let other_account_string = "a".to_string() + env::predecessor_account_id().as_str();
        // 文字列からAccountId型に変更して, other_accountによって使用中という状態を再現
        let mut bike = Bike::InUse(other_account_string.try_into().unwrap());
        bike.be_available();
    }
}
