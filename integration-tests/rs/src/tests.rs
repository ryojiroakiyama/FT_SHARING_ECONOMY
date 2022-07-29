use near_sdk::json_types::U128;
use near_units::{parse_near};
use serde_json::json;
use workspaces::prelude::*;
use workspaces::{network::Sandbox, Account, Contract, Worker, AccountId};

const BIKE_WASM_FILEPATH: &str = "../../out/main.wasm";
const FT_CONTRACT_ACCOUNT: &str = "my_ft.testnet";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initiate environemnt
    let worker = workspaces::sandbox().await?;
    
    // deploy bike contract
    let bike_wasm = std::fs::read(BIKE_WASM_FILEPATH)?;
    let bike_contract = worker.dev_deploy(&bike_wasm).await?;
    
    // pull ft contract
    let ft_contract = pull_contract(&worker).await?;
    
    // create user accounts
    let owner = worker.root_account().unwrap();
    let alice = owner
        .create_subaccount(&worker, "alice")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;

    // Initialize contracts
    ft_contract
        .call(&worker, "new_default_meta")
        .args_json(serde_json::json!({
            "owner_id": owner.id(),
            "total_supply": "100".to_string(),
        }))?
        .transact()
        .await?;
    bike_contract
        .call(&worker, "new")
        .args_json(serde_json::json!({
            "num_of_bikes": 5
        }))?
        .transact()
        .await?;
    bike_contract
        .as_account()
        .call(&worker, ft_contract.id(), "storage_deposit")
        .args_json(serde_json::json!({
            "account_id": bike_contract.id()
        }))?
        .deposit(1250000000000000000000)
        .gas(300000000000000)
        .transact()
        .await?;

    // begin tests
     test_init(&owner, &ft_contract, &bike_contract, &worker).await?;
     test_transfer_call_to_use_bike(&owner, &alice, &ft_contract, &bike_contract, &worker)
     .await?;
    Ok(())
}

async fn pull_contract(worker: &Worker<Sandbox>) -> anyhow::Result<Contract> {
    let testnet = workspaces::testnet_archival().await?;
    let contract_id: AccountId = FT_CONTRACT_ACCOUNT.parse()?;

    let contract = worker
        .import_contract(&contract_id, &testnet)
        .initial_balance(parse_near!("1000 N"))
        .transact()
        .await?;

    Ok(contract)
}

// 初期状態の確認
async fn test_init(
    owner: &Account,
    ft_contract: &Contract,
    bike_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    // バイクの数の確認
    let res: usize = owner
        .call(&worker, bike_contract.id(), "num_of_bikes")
        .args_json(json!({}))?
        .transact()
        .await?
        .json()?;
    assert_eq!(res, 5);
    // ownerの残高の確認
    let owner_balance: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id": owner.id()}))?
        .transact()
        .await?
        .json()?;
    assert_eq!(owner_balance.0, 100);
    println!("      Passed ✅ test_init");
    Ok(())
}

async fn test_transfer_call_to_use_bike(
    owner: &Account,
    user: &Account,
    ft_contract: &Contract,
    bike_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let user_initial_amount = 100;
    let transfer_amount = 30;
    let test_index = 0;

    // user, storage registory
    user.call(&worker, ft_contract.id(), "storage_deposit")
        .args_json(serde_json::json!({
            "account_id": user.id()
        }))?
        .deposit(1250000000000000000000)
        .gas(300000000000000)
        .transact()
        .await?;

    // userのFTの用意
    // FTコントラクトからユーザにFTを送信
    owner
        .call(&worker, ft_contract.id(), "ft_transfer")
        .args_json(serde_json::json!({
            "receiver_id": user.id(),
            "amount": user_initial_amount.to_string()
        }))?
        .deposit(1)
        .transact()
        .await?;

    // ft_transfer_callの呼び出し
    // bike_contractへ30FT送信し, バイクの使用を申請します
    user.call(&worker, ft_contract.id(), "ft_transfer_call")
        .args_json(serde_json::json!({
            "receiver_id": bike_contract.id(),
            "amount": transfer_amount.to_string(),
            "msg": test_index.to_string(),
        }))?
        .deposit(1)
        .gas(300000000000000)
        .transact()
        .await?;

    // test_indexのバイクの使用者がuseであるか確認
    let bike_user_id: AccountId = bike_contract
        .call(&worker, "who_is_using")
        .args_json(json!({"index": test_index}))?
        .transact()
        .await?
        .json()?;
    assert_eq!(user.id().clone(), bike_user_id);

    // userの残高の確認
    let user_balance: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id": user.id()}))?
        .transact()
        .await?
        .json()?;
    assert_eq!(user_balance.0, user_initial_amount - transfer_amount);

    // bike_contractの残高の確認
    let bike_contract: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id": bike_contract.id()}))?
        .transact()
        .await?
        .json()?;
    assert_eq!(bike_contract.0, transfer_amount);

    println!("      Passed ✅ test_transfer_call_to_use_bike");
    Ok(())
}