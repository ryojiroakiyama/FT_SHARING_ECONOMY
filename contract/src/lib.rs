use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env, log, near_bindgen,
    serde::{Deserialize, Serialize},
    AccountId,
};

//TODO: 追加機能集
//TODO: ユーザ1人一つしか使用できないようにする
//TODO: アカウント所持者はバイクの数を増やせる(事前にアカウントのIDを所持しておく)

// TODO: ユーザ間送金機能つける

//TODO: pubの付け方

// Define the default message
const DEFAULT_MESSAGE: &str = "Hello";
const NUMBER_OF_BIKES: usize = 5;

//TODO: いらないderiveを消す
// Bikeの状態
// enumでの管理: 状態遷移が明瞭, かつ必ずこの内のどれかの状態であるという保証ができる利点があると理解
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum Bike {
    Available,             // 使用可能
    InUse(AccountId),      // AccountIdによって使用中
    Inspection(AccountId), // AccountIdによって点検中
}

//TODO: これいるのか
// デフォルトでは使用可能状態
impl Default for Bike {
    fn default() -> Self {
        Bike::Available
    }
}

//TODO: enumのメソッド消す
// Bikeに関する機能をメソッドでまとめる
impl Bike {
    fn is_available(&self) -> bool {
        self == &Bike::Available
    }

    // TODO: 本当にoptionがnull or valueになるかは定義を調べられていない
    fn who_is_using(&self) -> Option<AccountId> {
        match self {
            Bike::InUse(account_id) => Some(account_id.clone()),
            _ => None,
        }
    }

    fn who_is_inspecting(&self) -> Option<AccountId> {
        match self {
            Bike::Inspection(account_id) => Some(account_id.clone()),
            _ => None,
        }
    }

    // 使用可能に状態を変更
    // 変更ができない場合はなるべく早くpanicを起こすことでトランザクション時間を短くする
    fn be_available(&mut self) {
        //predecessor_account_id(): このコントラクトを呼び出しているアカウントを取得
        let caller = env::predecessor_account_id();
        match self {
            Bike::Available => panic!("Already available"),
            Bike::InUse(account_id) => {
                assert_eq!(account_id.clone(), caller, "Wrong account");
                *self = Bike::Available
            }
            Bike::Inspection(account_id) => {
                assert_eq!(account_id.clone(), caller, "Wrong account");
                *self = Bike::Available
            }
        };
    }

    // 使用中に状態を変更
    fn be_in_use(&mut self) {
        assert!(self.is_available(), "Not available");
        *self = Bike::InUse(env::predecessor_account_id());
    }

    // 点検中に状態を変更
    fn be_inspection(&mut self) {
        assert!(self.is_available(), "Not available");
        *self = Bike::Inspection(env::predecessor_account_id());
    }
}

// Bikeの情報をフロントエンドへ送信する(Json形式へSerialize)際に使用する構造体
// フロント側で理解しやすい構造体に整形した方が開発が楽だと判断したので用意
#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonBike {
    available: bool,
    in_use: bool,
    used_by: Option<AccountId>,
    inspection: bool,
    inspected_by: Option<AccountId>,
}

impl Default for JsonBike {
    fn default() -> Self {
        Self {
            available: false,
            in_use: false,
            used_by: None,
            inspection: false,
            inspected_by: None,
        }
    }
}

// コントラクトの定義
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    message: String,
    bikes: Vec<Bike>,
}

//TODO: initに変更して, 指定した数のsizeでvector作る, DefaultOnPanicにする
// TODO: 引数でアカウントIDをもらって保存するようにする
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

    // TODO: 関数名, 構造体名変える
    // 各バイクが使用可能かどうかをベクターで返却
    pub fn get_json_bikes(&self) -> Vec<JsonBike> {
        self.bikes
            .iter()
            .map(|bike| {
                // デフォルトでは全てがfalse or None
                let mut json_bike = JsonBike::default();
                // bikeの状態によって各変数を編集する
                match bike {
                    Bike::Available => json_bike.available = true,
                    Bike::InUse(account_id) => {
                        json_bike.in_use = true;
                        json_bike.used_by = Some(account_id.clone());
                    }
                    Bike::Inspection(account_id) => {
                        json_bike.inspection = true;
                        json_bike.inspected_by = Some(account_id.clone());
                    }
                };
                json_bike
            })
            .collect()
    }

    // 誰がバイクを使用中かを返却
    pub fn who_is_using(&self, index: usize) -> Option<AccountId> {
        self.bikes[index].who_is_using()
    }

    // 誰がバイクを点検中かを返却
    pub fn who_is_inspecting(&self, index: usize) -> Option<AccountId> {
        self.bikes[index].who_is_inspecting()
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
            assert!(bike.is_available());
        }
    }

    #[test]
    fn set_then_get_greeting() {
        let mut contract = Contract::default();
        contract.set_greeting("howdy".to_string());
        assert_eq!(contract.get_greeting(), "howdy".to_string());
    }

    // メソッドを呼び出しているアカウントの取得
    // デフォルトでは"bob.testnet"みたいです
    fn caller() -> AccountId {
        env::predecessor_account_id()
    }

    // caller()と別のアカウントを作成
    fn another_caller() -> AccountId {
        // caller()に接頭語"a"をつけて, 別のアカウントを表す文字列作成
        let another_account_string = "a".to_string() + caller().as_str();
        // 文字列からAccountId型に変更
        another_account_string.try_into().unwrap()
    }

    // バイクの状態を変更して, bikeの状態を確認
    #[test]
    fn change_state_then_get_states() {
        let mut contract = Contract::default();

        // 初期状態をチェック
        for is_available in contract.get_json_bikes() {
            assert!(is_available);
        }

        let idx_to_check = contract.bikes.len() - 1;

        // バイクを使用, 状態をチェック
        contract.use_bike(idx_to_check);
        // バイクを使用したユーザからみた情報
        for (i, is_available) in contract.get_json_bikes().iter().enumerate() {
            if i == idx_to_check {
                assert!(!is_available);
                assert_eq!(contract.bikes[i].who_is_using().unwrap(), caller())
            } else {
                assert!(is_available);
            }
        }
        // 他のアカウントから見た情報
        for (i, is_available) in contract.get_json_bikes().iter().enumerate() {
            if i == idx_to_check {
                assert!(!is_available);
            } else {
                assert!(is_available);
            }
        }

        // バイクを返却, 状態をチェック
        contract.return_bike(idx_to_check);
        for is_available in contract.get_json_bikes() {
            assert!(is_available);
        }

        // バイクを点検, 状態をチェック
        contract.inspect_bike(idx_to_check);
        // バイクを使用したユーザからみた情報
        for (i, is_available) in contract.get_json_bikes().iter().enumerate() {
            if i == idx_to_check {
                assert!(!is_available);
                assert_eq!(contract.bikes[i].who_is_inspecting().unwrap(), caller())
            } else {
                assert!(is_available);
            }
        }
        // 他のアカウントから見た情報
        for (i, is_available) in contract.get_json_bikes().iter().enumerate() {
            if i == idx_to_check {
                assert!(!is_available);
            } else {
                assert!(is_available);
            }
        }

        // バイクを返却, 状態をチェック
        contract.return_bike(idx_to_check);
        for is_available in contract.get_json_bikes() {
            assert!(is_available);
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
        assert_eq!(bike.who_is_using().unwrap(), caller());

        // バイク返却, 状態チェック
        bike.be_available();
        assert!(bike.is_available());

        // バイク点検, 状態チェック
        bike.be_inspection();
        assert_eq!(bike.who_is_inspecting().unwrap(), caller());

        // バイク返却, 状態チェック
        bike.be_available();
        assert!(bike.is_available());
    }

    // 重複してバイクを使用->パニックを起こすか確認
    #[test]
    #[should_panic(expected = "Not available")]
    fn duplicate_use() {
        let mut bike = Bike::InUse(caller());
        bike.be_in_use();
    }

    // 重複してバイクを点検->パニックを起こすか確認
    #[test]
    #[should_panic(expected = "Not available")]
    fn duplicate_inspect() {
        let mut bike = Bike::Inspection(caller());
        bike.be_inspection();
    }

    // 重複してバイクを使用可能に->パニックを起こすか確認
    #[test]
    #[should_panic(expected = "Already available")]
    fn duplicate_return() {
        let mut bike = Bike::Available;
        bike.be_available();
    }

    // 別のアカウントが使用中に使用可能に変更->パニックを起こすか確認
    #[test]
    #[should_panic(expected = "Wrong account")]
    fn return_by_other_account() {
        let mut bike = Bike::InUse(another_caller());
        bike.be_available();
    }
}
