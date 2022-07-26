use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env, ext_contract,
    json_types::U128,
    log, near_bindgen, AccountId, Gas, PanicOnDefault, PromiseOrValue, PromiseResult,
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
// panicやassertの使用について: 処理ができない場合はなるべく早くプログラムを停止させることでトランザクションにかかる余分なガス代を削減するため.
// 各env::~idの説明を入れる
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

    // FTコントラクトのft_transfer_call()が呼び出された際に実行するメソッド
    // 30FTの受信を確認して, use_bikeメソッドを呼び出しバイクを使用中に変更します.
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
        // 受信したFTは全て受け取るので0を返却.
        PromiseOrValue::Value(U128::from(0))
    }

    // バイク 使用可 -> 使用中
    // ft_on_transferによって呼び出されます.
    fn use_bike(&mut self, index: usize) {
        // env::signer_account_id(): FTコントラクトのft_transfer_call()を呼び出しているアカウントを取得
        let user_id = env::signer_account_id();
        log!("{} uses bike", &user_id);
        match &self.bikes[index] {
            Bike::Available => self.bikes[index] = Bike::InUse(user_id),
            _ => panic!("Bike is not available"),
        }
    }

    // バイク 使用可 -> 点検中
    pub fn inspect_bike(&mut self, index: usize) {
        // env::predecessor_account_id(): このメソッドを呼び出しているアカウントを取得
        let inspector_id = env::predecessor_account_id();
        log!("{} inspects bike", &inspector_id);
        match &self.bikes[index] {
            Bike::Available => self.bikes[index] = Bike::Inspection(inspector_id),
            _ => panic!("Bike is not available"),
        }
    }

    // バイク 使用中or点検中 -> 使用可
    pub fn return_bike(&mut self, index: usize) {
        // env::predecessor_account_id(): このメソッドを呼び出しているアカウントを取得
        let predecessor = env::predecessor_account_id();
        log!("{} returns bike", &predecessor);
        match &self.bikes[index] {
            Bike::Available => panic!("Bike is already available"),
            Bike::InUse(user) => {
                assert_eq!(user.clone(), predecessor, "Fail due to wrong account");
                self.bikes[index] = Bike::Available
            }
            Bike::Inspection(inspector) => {
                assert_eq!(inspector.clone(), predecessor, "Fail due to wrong account");
                Self::cross_contract_call_transfer(index);
            }
        };
    }

    // cross contract call
    // FTコントラクトのft_transferメソッドを呼び出し(cross contract call),
    // 点検をしてくれたユーザのアカウントへ報酬として15FTを送信します.
    pub fn cross_contract_call_transfer(index: usize) {
        let contract_id = "my_ft.testnet".parse().unwrap();
        let amount = "15".to_string();
        let receiver_id = env::predecessor_account_id().to_string();
        let gas = Gas(3_000_000_000_000);

        log!(
            "{} transfer to {}: {} FT",
            env::current_account_id(),
            &receiver_id,
            &amount
        );

        // cross contract call
        // callback関数としてバイクを返却するcallback_return_bikeメソッドを呼び出します.
        ext_ft::ext(contract_id)
            .with_attached_deposit(1)
            .ft_transfer(receiver_id, amount, None)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(gas)
                    .callback_return_bike(index),
            );
    }

    // callback
    // cross_contract_call_reward_to_inspectorメソッドの実行後に実行するメソッドを定義
    #[private]
    pub fn callback_return_bike(&mut self, index: usize) {
        assert_eq!(env::promise_results_count(), 1, "This is a callback method");
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => panic!("Fail cross-contract call"),
            // 成功時のみBikeを返却(使用可能に変更)
            PromiseResult::Successful(_) => self.bikes[index] = Bike::Available,
        }
    }
}

// トランザクションの実行環境をシュミレーション
#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    // テスト環境の構築に必要なものをインポート
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;

    // Contractのモジュールをインポート
    use super::*;

    // VMContextBuilderのテンプレートを用意
    // VMContextBuilder: テスト環境(モックされたブロックチェーン)をcontext(テスト材料)をもとに変更できるインターフェース
    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    // newメソッドのテスト
    #[test]
    fn test_new() {
        // コンテクスト用意
        let mut context = get_context(accounts(1));
        // テスト環境を初期化
        testing_env!(context.build());
        let init_num = 5;
        let contract = Contract::new(init_num);

        // view関数のみ実行する環境に初期化
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.num_of_bikes(), init_num);
        for i in 0..init_num {
            assert!(contract.is_available(i))
        }
    }

    // use_bike(), who_is_using()のテスト
    #[test]
    fn check_using_account() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = Contract::new(5);

        // チェックに使用するindexを定義
        let test_index = contract.bikes.len() - 1;
        // バイクを使用
        contract.use_bike(test_index);

        testing_env!(context.is_view(true).build());
        // バイクの状態をチェック
        for i in 0..contract.num_of_bikes() {
            if i == test_index {
                assert_eq!(accounts(1), contract.who_is_using(i).unwrap());
            } else {
                assert!(contract.is_available(i))
            }
        }
    }

    // inspect_bike(), who_is_inspecting()のテスト
    #[test]
    fn check_inspecting_account() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = Contract::new(5);

        // チェックに使用するindexを定義
        let test_index = contract.bikes.len() - 1;
        // バイクを点検
        contract.inspect_bike(test_index);

        testing_env!(context.is_view(true).build());
        // バイクの状態をチェック
        for i in 0..contract.num_of_bikes() {
            if i == test_index {
                assert_eq!(accounts(1), contract.who_is_inspecting(i).unwrap());
            } else {
                assert!(contract.is_available(i))
            }
        }
    }

    // 重複してバイクを点検->パニックを起こすか確認
    #[test]
    #[should_panic(expected = "Bike is not available")]
    fn duplicate_use() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = Contract::new(5);

        contract.use_bike(0);
        contract.use_bike(0);
    }

    // 重複してバイクを点検->パニックを起こすか確認
    #[test]
    #[should_panic(expected = "Bike is not available")]
    fn duplicate_inspect() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = Contract::new(5);

        contract.inspect_bike(0);
        contract.inspect_bike(0);
    }

    // 使用可能状態のバイクを返却->パニックを起こすか確認
    #[test]
    #[should_panic(expected = "Bike is already available")]
    fn duplicate_return() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = Contract::new(5);

        contract.return_bike(0);
    }

    // 別のアカウントが使用中に使用可能に変更->パニックを起こすか確認
    #[test]
    #[should_panic(expected = "Fail due to wrong account")]
    fn return_by_other_account() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = Contract::new(5);

        // accounts(1)がバイクを使用
        contract.use_bike(0);

        // accounts(2)でバイクを使用
        testing_env!(context.predecessor_account_id(accounts(2)).build());
        contract.return_bike(0);
    }
}
