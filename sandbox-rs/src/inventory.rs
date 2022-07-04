use serde_json::json;
use serde::{Deserialize, Serialize};
use near_units::{parse_gas, parse_near};
use workspaces::{Account, AccountId, Contract, Network, Worker};
use workspaces::prelude::*;
use common::*;

mod common;

const BOOST_FILEPATH: &str = "./res/boost_farming.wasm";
const FT_FILEPATH: &str = "./res/mock_ft.wasm";
const NFT_FILEPATH: &str = "./res/mock_mft.wasm";


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let boost_wasm = std::fs::read(BOOST_FILEPATH)?;
    let ft_wasm = std::fs::read(FT_FILEPATH)?;
    let nft_wasm = std::fs::read(NFT_FILEPATH)?;
    let boost = worker.dev_deploy(&boost_wasm).await?;
    let ft = worker.dev_deploy(&ft_wasm).await?;
    let nft = worker.dev_deploy(&nft_wasm).await?;

    let owner = worker.root_account();
    
    let user1 = owner
        .create_subaccount(&worker, "user1")
        .initial_balance(parse_near!("100 N"))
        .transact()
        .await?
        .result;
    
        // Create a root croncat account with agent subaccounts to schedule tasks.
    let user2 = worker.dev_create_account().await?;

    let outcome = boost
        .call(&worker, "new")
        .args_json(json!({
                "owner_id": owner.id(),
        }))?
        .transact()
        .await?;
    println!("new: {:#?}\n", outcome);
    
    let result: serde_json::Value = worker
        .view(
            boost.id(), 
            "get_metadata", 
            Vec::new(),
        )
        .await?
        .json()?;
    
    println!("get_metadata: {}", result);    

    let md: Metadata = serde_json::from_value(result).unwrap();
    println!("get_metadata obj: {:#?}", md);
    assert_eq!(md.owner_id.to_string(), "test.near".to_string());
    assert_eq!(&md.owner_id, owner.id());

    Ok(())
}