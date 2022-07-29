use near_sdk::json_types::U128;
use near_units::{parse_gas, parse_near};
use serde_json::json;
use workspaces::prelude::*;
use workspaces::result::CallExecutionDetails;
use workspaces::{network::Sandbox, Account, Contract, Worker};

const BIKE_WASM_FILEPATH: &str = "../../out/main.wasm";
const FT_WASM_FILEPATH: &str = "../../fungible_token.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initiate environemnt
    let worker = workspaces::sandbox().await?;

    // deploy contracts
    let ft_wasm = std::fs::read(FT_WASM_FILEPATH)?;
    let ft_contract = worker.dev_deploy(&ft_wasm).await?;
    let bike_wasm = std::fs::read(BIKE_WASM_FILEPATH)?;
    let bike_contract = worker.dev_deploy(&bike_wasm).await?;

    // create accounts
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
            "total_supply": "1000000000000000".to_string(),
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

    // begin tests
     test_init(&owner, &ft_contract, &bike_contract, &worker).await?;
    // test_transfer_call_with_burned_amount(&owner, &charlie, &ft_contract, &defi_contract, &worker)
    // .await?;
    Ok(())
}

async fn test_init(
    owner: &Account,
    ft_contract: &Contract,
    bike_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let res: usize = owner
        .call(&worker, bike_contract.id(), "num_of_bikes")
        .args_json(json!({}))?
        .transact()
        .await?
        .json()?;
    assert_eq!(res, 5);
    let owner_balance: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id": owner.id()}))?
        .transact()
        .await?
        .json()?;
    assert_eq!(owner_balance.0, 1000000000000000);
    println!("      Passed ✅ test_init");
    Ok(())
}

//async fn test_transfer_call_with_burned_amount(
//    owner: &Account,
//    user: &Account,
//    ft_contract: &Contract,
//    defi_contract: &Contract,
//    worker: &Worker<Sandbox>,
//) -> anyhow::Result<()> {
//    let transfer_amount_str = parse_near!("1,000,000 N").to_string();
//    let ftc_amount_str = parse_near!("1,000 N").to_string();

//    // register user
//    owner
//        .call(&worker, ft_contract.id(), "storage_deposit")
//        .args_json(serde_json::json!({
//            "account_id": user.id()
//        }))?
//        .deposit(parse_near!("0.008 N"))
//        .transact()
//        .await?;

//    // transfer ft
//    owner
//        .call(&worker, ft_contract.id(), "ft_transfer")
//        .args_json(serde_json::json!({
//            "receiver_id": user.id(),
//            "amount": transfer_amount_str
//        }))?
//        .deposit(1)
//        .transact()
//        .await?;

//    user.call(&worker, ft_contract.id(), "ft_transfer_call")
//        .args_json(serde_json::json!({
//            "receiver_id": defi_contract.id(),
//            "amount": ftc_amount_str,
//            "msg": "0",
//        }))?
//        .deposit(1)
//        .gas(parse_gas!("200 Tgas") as u64)
//        .transact()
//        .await?;

//    let storage_result: CallExecutionDetails = user
//        .call(&worker, ft_contract.id(), "storage_unregister")
//        .args_json(serde_json::json!({"force": true }))?
//        .deposit(1)
//        .transact()
//        .await?;

//    // assert new state
//    assert_eq!(
//        storage_result.logs()[0],
//        format!(
//            "Closed @{} with {}",
//            user.id(),
//            parse_near!("999,000 N") // balance after defi ft transfer
//        )
//    );

//    let total_supply: U128 = owner
//        .call(&worker, ft_contract.id(), "ft_total_supply")
//        .args_json(json!({}))?
//        .transact()
//        .await?
//        .json()?;
//    assert_eq!(total_supply, U128::from(parse_near!("999,000,000 N")));

//    let defi_balance: U128 = owner
//        .call(&worker, ft_contract.id(), "ft_total_supply")
//        .args_json(json!({"account_id": defi_contract.id()}))?
//        .transact()
//        .await?
//        .json()?;
//    assert_eq!(defi_balance, U128::from(parse_near!("999,000,000 N")));

//    println!("      Passed ✅ test_transfer_call_with_burned_amount");
//    Ok(())
//}