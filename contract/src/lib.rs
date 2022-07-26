use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env, ext_contract,
    json_types::U128,
    log, near_bindgen, AccountId, PanicOnDefault, PromiseOrValue,
};

#[ext_contract(ext_ft)]
trait FungibleToken {
    fn ft_transfer(&mut self, receiver_id: String, amount: String, memo: Option<String>);
}

// Bikeの状態をenumで管理します.
// enumでの管理: 状態遷移が明瞭, かつ必ずこの内のどれかの状態であるという保証ができる利点があると理解
#[derive(BorshDeserialize, BorshSerialize)]
enum Bike {
    Available,             // 使用可能
    InUse(AccountId),      // AccountIdによって使用中
    Inspection(AccountId), // AccountIdによって点検中
}

// コントラクトの定義
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    bikes: Vec<Bike>,
}

// Implement the contract structure
#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(num_of_bikes: usize) -> Self {
        log!("initialize Contract with {} bikes", num_of_bikes);
        Self {
            bikes: {
                let mut bikes = Vec::new();
                for _i in 0..num_of_bikes {
                    bikes.push(Bike::Available);
                }
                bikes
            },
        }
    }

    pub fn num_of_bikes(&self) -> usize {
        self.bikes.len()
    }

    pub fn is_available(&self, index: usize) -> bool {
        match self.bikes[index] {
            Bike::Available => true,
            _ => false,
        }
    }

    pub fn who_is_using(&self, index: usize) -> Option<AccountId> {
        match &self.bikes[index] {
            Bike::InUse(user_id) => Some(user_id.clone()),
            _ => None,
        }
    }

    pub fn who_is_inspecting(&self, index: usize) -> Option<AccountId> {
        match &self.bikes[index] {
            Bike::Inspection(inspector_id) => Some(inspector_id.clone()),
            _ => None,
        }
    }

    // 以下バイクの状態を変更するメソッドを定義します.
    // panicやassertの使用について: 処理ができない場合はなるべく早くプログラムを停止させることでトランザクションにかかる余分なガス代を削減するため.

    // メソッドを使う
    // 使用可 -> 使用中
    pub fn use_bike(&mut self, index: usize) {
        log!("use_bike");
        match &self.bikes[index] {
            Bike::Available => self.bikes[index] = Bike::InUse(env::signer_account_id()),
            _ => panic!("Bike is not available"),
        }
    }

    // 使用可 -> 点検中
    pub fn inspect_bike(&mut self, index: usize) {
        log!("inspect_bike");
        match &self.bikes[index] {
            Bike::Available => self.bikes[index] = Bike::Inspection(env::predecessor_account_id()),
            _ => panic!("Bike is not available"),
        }
    }

    // 使用中or点検中 -> 使用可
    pub fn return_bike(&mut self, index: usize) {
        log!("return_bike");
        //predecessor_account_id(): このコントラクトを呼び出しているアカウントを取得
        let predecessor = env::predecessor_account_id();
        match &self.bikes[index] {
            Bike::Available => panic!("Bike is already available"),
            Bike::InUse(user) => {
                assert_eq!(user.clone(), predecessor, "Fail due to wrong account");
                self.bikes[index] = Bike::Available
            }
            Bike::Inspection(inspector) => {
                assert_eq!(inspector.clone(), predecessor, "Fail due to wrong account");
                Self::ft_transfer(
                    "my_ft.testnet".parse().unwrap(),
                    "15".to_string(),
                    env::predecessor_account_id(),
                );
                self.bikes[index] = Bike::Available
            }
        };
    }

    //　TODO:エラー時にフロント側で無言なのが困る
    // msgでの関数の切り替えなどのできるかも？もう一つの引数か？
    pub fn ft_on_transfer(
        &mut self,
        sender_id: String,
        amount: String,
        msg: String,
    ) -> PromiseOrValue<U128> {
        assert_eq!(amount, "30", "Require 30FT to use the bike");
        log!(
            "in ft_on_transfer: sender:{}, amount:{}, msg:{}",
            sender_id,
            amount,
            msg
        );
        self.use_bike(msg.parse().unwrap());
        PromiseOrValue::Value(U128::from(0))
    }

    //TODO: エラー時にエラーを拾えるか？
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
    fn check_default() {
        let init_num = 5;
        let contract = Contract::new(init_num);
        assert_eq!(contract.num_of_bikes(), init_num);
        for i in 0..init_num {
            assert!(contract.is_available(i))
        }
    }

    // メソッドを呼び出しているアカウントの取得
    // デフォルトでは"bob.testnet"となっています
    fn predecessor() -> AccountId {
        env::predecessor_account_id()
    }

    // predecessor()と別のアカウントを作成
    fn another() -> AccountId {
        // predecessor()に接頭語"a"をつけて, 別のアカウントを表す文字列作成
        let another_account_string = "a".to_string() + predecessor().as_str();
        // 文字列からAccountId型に変更
        another_account_string.try_into().unwrap()
    }

    // バイクの状態を変更して, bikeの状態を確認
    //#[test]
    fn change_state_then_get_states() {
        let mut contract = Contract::new(5);
        let test_index = contract.bikes.len() - 1;

        // バイクを使用, 状態をチェック
        contract.use_bike(test_index);
        for i in 0..contract.num_of_bikes() {
            if i == test_index {
                assert_eq!(predecessor(), contract.who_is_using(i).unwrap());
            } else {
                assert!(contract.is_available(i))
            }
        }

        // バイクを返却, 状態をチェック
        contract.return_bike(test_index);
        for i in 0..contract.num_of_bikes() {
            assert!(contract.is_available(i))
        }

        // バイクを点検, 状態をチェック
        contract.use_bike(test_index);
        for i in 0..contract.num_of_bikes() {
            if i == test_index {
                assert_eq!(predecessor(), contract.who_is_inspecting(i).unwrap());
            } else {
                assert!(contract.is_available(i))
            }
        }

        // バイクを返却, 状態をチェック
        contract.return_bike(test_index);
        for i in 0..contract.num_of_bikes() {
            assert!(contract.is_available(i))
        }
    }

    //TODO:duplicate ft_transfer

    // 重複してバイクを点検->パニックを起こすか確認
    #[test]
    #[should_panic(expected = "Bike is not available")]
    fn duplicate_use() {
        let mut contract = Contract::new(5);
        let test_index = 0;
        contract.inspect_bike(test_index);
        contract.inspect_bike(test_index);
    }

    // 重複してバイクを返却->パニックを起こすか確認
    #[test]
    #[should_panic(expected = "Bike is already available")]
    fn duplicate_return() {
        let mut contract = Contract::new(5);
        contract.return_bike(0);
    }

    // 別のアカウントが使用中に使用可能に変更->パニックを起こすか確認
    #[test]
    #[should_panic(expected = "Fail due to wrong account")]
    fn return_by_other_account() {
        let mut contract = Contract::new(5);
        let test_index = contract.bikes.len() - 1;
        // 別のアカウントによる使用中に設定
        contract.bikes[test_index] = Bike::InUse(another());
        // 別のアカウントが使用中のバイクを返却
        contract.return_bike(test_index);
    }
}
