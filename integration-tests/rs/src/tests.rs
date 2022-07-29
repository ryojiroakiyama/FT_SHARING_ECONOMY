use near_sdk::json_types::U128;
use near_units::{parse_near};
use serde_json::json;
use workspaces::prelude::*;
use workspaces::{network::Sandbox, Account, Contract, Worker, AccountId};

const BIKE_WASM_FILEPATH: &str = "../../out/main.wasm";
const FT_CONTRACT_ACCOUNT: &str = "my_ft.testnet";

const FT_TOTAL_SUPPLY: &str = "1000";

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
        .initial_balance(parse_near!("100 N"))
        .transact()
        .await?
        .into_result()?;
    let bob = owner
        .create_subaccount(&worker, "bob")
        .initial_balance(parse_near!("100 N"))
        .transact()
        .await?
        .into_result()?;

    // Initialize contracts
    ft_contract
        .call(&worker, "new_default_meta")
        .args_json(serde_json::json!({
            "owner_id": owner.id(),
            "total_supply": FT_TOTAL_SUPPLY.to_string(),
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
     test_transfer_call_to_use_bike(&owner, &alice, &ft_contract, &bike_contract, &worker).await?;
     test_transfer_ft_to_user_inspected_bike(&owner, &bob, &ft_contract, &bike_contract, &worker).await?;
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
    assert_eq!(owner_balance.0, FT_TOTAL_SUPPLY.parse::<u128>().unwrap());
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
    let test_bike_index = 0;

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
            "msg": test_bike_index.to_string(),
        }))?
        .deposit(1)
        .gas(300000000000000)
        .transact()
        .await?;

    // test_bike_indexのバイクの使用者がuserであるか確認
    let bike_user_id: AccountId = bike_contract
        .call(&worker, "who_is_using")
        .args_json(json!({"index": test_bike_index}))?
        .transact()
        .await?
        .json()?;
    assert_eq!(user.id().clone(), bike_user_id);

    // return_bike()の呼び出し
    user.call(&worker, bike_contract.id(), "return_bike")
        .args_json(serde_json::json!({
            "index": test_bike_index,
        }))?
        .gas(300000000000000)
        .transact()
        .await?;

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


async fn test_transfer_ft_to_user_inspected_bike(
    owner: &Account,
    user: &Account,
    ft_contract: &Contract,
    bike_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let remuneration_amount = 15;
    let test_bike_index = 0;

    // user, storage registory
    user.call(&worker, ft_contract.id(), "storage_deposit")
        .args_json(serde_json::json!({
            "account_id": user.id()
        }))?
        .deposit(1250000000000000000000)
        .gas(300000000000000)
        .transact()
        .await?;

    // bikeコントラクトのFTの用意
    // FTコントラクトからFTを送信
    owner
        .call(&worker, ft_contract.id(), "ft_transfer")
        .args_json(serde_json::json!({
            "receiver_id": bike_contract.id(),
            "amount": "50".to_string()
        }))?
        .deposit(1)
        .transact()
        .await?;

    // この時点でのユーザの残高確認
    let user_balance: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id": user.id()}))?
        .transact()
        .await?
        .json()?;
    assert_eq!(user_balance.0, 0);

    // inspect_bike()の呼び出し
    user.call(&worker, bike_contract.id(), "inspect_bike")
        .args_json(serde_json::json!({
            "index": test_bike_index,
        }))?
        .gas(300000000000000)
        .transact()
        .await?;

    // この時点でのユーザの残高確認
    let user_balance: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id": user.id()}))?
        .transact()
        .await?
        .json()?;
    assert_eq!(user_balance.0, 0);

    // return_bike()の呼び出し
    user.call(&worker, bike_contract.id(), "return_bike")
        .args_json(serde_json::json!({
            "index": test_bike_index,
        }))?
        .gas(300000000000000)
        .transact()
        .await?;

    // ユーザの残高が増えていることを確認
    let user_balance: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id": user.id()}))?
        .transact()
        .await?
        .json()?;
    assert_eq!(user_balance.0, remuneration_amount);

    println!("      Passed ✅ test_transfer_ft_to_user_inspected_bike");
    Ok(())
}