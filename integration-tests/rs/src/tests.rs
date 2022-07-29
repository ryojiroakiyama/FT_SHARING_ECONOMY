use near_sdk::json_types::U128;
use near_units::{parse_gas, parse_near};
use serde_json::json;
use workspaces::prelude::*;
use workspaces::result::CallExecutionDetails;
use workspaces::{network::Sandbox, Account, Contract, Worker};

const DEFI_WASM_FILEPATH: &str = "../../out/main.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initiate environemnt
    let worker = workspaces::sandbox().await?;

    // deploy contracts
    let wasm = std::fs::read(DEFI_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(&wasm).await?;

    // create accounts
    let owner = worker.root_account().unwrap();
    let alice = owner
        .create_subaccount(&worker, "alice")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;

    // Initialize contracts
    contract
        .call(&worker, "new")
        .args_json(serde_json::json!({
            "num_of_bikes": 5
        }))?
        .transact()
        .await?;

    // begin tests
     test_init(&owner, &contract, &worker).await?;
    Ok(())
}

 async fn test_init(
     owner: &Account,
     contract: &Contract,
     worker: &Worker<Sandbox>,
 ) -> anyhow::Result<()> {
     let res: usize = owner
         .call(&worker, contract.id(), "num_of_bikes")
         .args_json(json!({}))?
         .transact()
         .await?
         .json()?;
     assert_eq!(res, 5);
     println!("      Passed ✅ test_init");
     Ok(())
 }