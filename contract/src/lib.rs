use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env, ext_contract,
    json_types::U128,
    log, near_bindgen,
    serde::Serialize,
    AccountId, PromiseOrValue,
};

//TODO: storage系はftの関数ではない？？storage_managementについてもう一度読む
// https://nomicon.io/Standards/StorageManagement これのコード内コメントのところ
#[ext_contract(ext_ft)]
trait FungibleToken {
    // change methods
    fn ft_transfer(&mut self, receiver_id: String, amount: String, memo: Option<String>);
    fn ft_transfer_call(
        &mut self,
        receiver_id: String,
        amount: String,
        memo: Option<String>,
        msg: String,
    ) -> U128;

    // view methods
    fn ft_total_supply(&self) -> String;
    fn ft_balance_of(&self, account_id: String) -> String;
}

//TODO: 追加機能集
//TODO: ユーザ1人一つしか使用できないようにする
//TODO: アカウント所持者はバイクの数を増やせる(事前にアカウントのIDを所持しておく)

const NUMBER_OF_BIKES: usize = 5;

// Bikeの状態
// enumでの管理: 状態遷移が明瞭, かつ必ずこの内のどれかの状態であるという保証ができる利点があると理解
#[derive(BorshDeserialize, BorshSerialize)]
enum Bike {
    Available,             // 使用可能
    InUse(AccountId),      // AccountIdによって使用中
    Inspection(AccountId), // AccountIdによって点検中
}

// Bikeの情報をフロントエンドへ送信する(Json形式へSerialize)際に使用する構造体
// フロント側で理解しやすいデータ型を用意した方が全体の開発が楽だと判断したので用意
#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonBike {
    available: bool,
    in_use: bool,
    used_by: Option<AccountId>,
    inspection: bool,
    inspected_by: Option<AccountId>,
}

// コントラクトの定義
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    bikes: Vec<Bike>,
}

//　TODO: initに変更して, 指定した数のsizeでvector作る, DefaultOnPanicにする
// TODO: 引数でアカウントIDをもらって保存するようにする
impl Default for Contract {
    fn default() -> Self {
        log!("initialize Contract");
        Self {
            bikes: {
                let mut v = Vec::new();
                let mut index = 0;
                while index < NUMBER_OF_BIKES {
                    v.push(Bike::Available);
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
    // 各バイクの情報をJsonBikeのベクターで返却
    pub fn get_bikes(&self) -> Vec<JsonBike> {
        log!("get_bikes");
        self.bikes
            .iter()
            .map(|bike| {
                // 全てをfalse or Noneで用意
                let mut json_bike = JsonBike {
                    available: false,
                    in_use: false,
                    used_by: None,
                    inspection: false,
                    inspected_by: None,
                };
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

    // 以下バイクの状態を変更するメソッド
    // panicやassertの使用について: 処理ができない場合はなるべく早くプログラムを停止させることでトランザクションにかかる余分なガス代を削減するため

    // 使用可 -> 使用中
    pub fn use_bike(&mut self, index: usize) {
        log!("use_bike");
        match &self.bikes[index] {
            Bike::Available => self.bikes[index] = Bike::InUse(env::signer_account_id()),
            _ => panic!("Not available"),
        }
    }

    // 使用可 -> 点検中
    pub fn inspect_bike(&mut self, index: usize) {
        log!("inspect_bike");
        match &self.bikes[index] {
            Bike::Available => self.bikes[index] = Bike::Inspection(env::predecessor_account_id()),
            _ => panic!("Not available"),
        }
    }

    //TODO:エラー文丁寧に
    // 使用中or点検中 -> 使用可
    pub fn return_bike(&mut self, index: usize) {
        log!("return_bike");
        //predecessor_account_id(): このコントラクトを呼び出しているアカウントを取得
        let caller = env::predecessor_account_id();
        match &self.bikes[index] {
            Bike::Available => panic!("Already available"),
            Bike::InUse(account_id) => {
                assert_eq!(account_id.clone(), caller, "Wrong account");
                self.bikes[index] = Bike::Available
            }
            Bike::Inspection(account_id) => {
                assert_eq!(account_id.clone(), caller, "Wrong account");
                Self::ft_transfer(
                    "my_ft.testnet".parse().unwrap(),
                    "15".to_string(),
                    env::predecessor_account_id(),
                );
                self.bikes[index] = Bike::Available
            }
        };
    }

    //TODO: 30支払われたかの確認を入れる
    pub fn ft_on_transfer(
        &mut self,
        sender_id: String,
        amount: String,
        msg: String,
    ) -> PromiseOrValue<U128> {
        log!(
            "in ft_on_transfer: sender:{}, amount:{}, msg:{}",
            sender_id,
            amount,
            msg
        );
        self.use_bike(msg.parse().unwrap());
        PromiseOrValue::Value(U128::from(0))
    }

    pub fn ft_transfer(contract: AccountId, amount: String, receiver: AccountId) {
        ext_ft::ext(contract).with_attached_deposit(1).ft_transfer(
            receiver.to_string(),
            amount,
            None,
        );
        log!("{} transfer to {}", env::current_account_id(), receiver);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_default() {
        let contract = Contract::default();
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
        for is_available in contract.get_bikes() {
            assert!(is_available);
        }

        let idx_to_check = contract.bikes.len() - 1;

        // バイクを使用, 状態をチェック
        contract.use_bike(idx_to_check);
        // バイクを使用したユーザからみた情報
        for (i, is_available) in contract.get_bikes().iter().enumerate() {
            if i == idx_to_check {
                assert!(!is_available);
                assert_eq!(contract.bikes[i].who_is_using().unwrap(), caller())
            } else {
                assert!(is_available);
            }
        }
        // 他のアカウントから見た情報
        for (i, is_available) in contract.get_bikes().iter().enumerate() {
            if i == idx_to_check {
                assert!(!is_available);
            } else {
                assert!(is_available);
            }
        }

        // バイクを返却, 状態をチェック
        contract.return_bike(idx_to_check);
        for is_available in contract.get_bikes() {
            assert!(is_available);
        }

        // バイクを点検, 状態をチェック
        contract.inspect_bike(idx_to_check);
        // バイクを使用したユーザからみた情報
        for (i, is_available) in contract.get_bikes().iter().enumerate() {
            if i == idx_to_check {
                assert!(!is_available);
                assert_eq!(contract.bikes[i].who_is_inspecting().unwrap(), caller())
            } else {
                assert!(is_available);
            }
        }
        // 他のアカウントから見た情報
        for (i, is_available) in contract.get_bikes().iter().enumerate() {
            if i == idx_to_check {
                assert!(!is_available);
            } else {
                assert!(is_available);
            }
        }

        // バイクを返却, 状態をチェック
        contract.return_bike(idx_to_check);
        for is_available in contract.get_bikes() {
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
