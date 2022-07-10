use common::*;
use serde_json::json;
use workspaces::prelude::*;
use workspaces::{Account, AccountId, Contract, Network, Worker};
use near_units::{parse_gas, parse_near};

mod common;

const BOOST_FILEPATH: &str = "./res/boost_farming.wasm";

/// test pour
async fn test_pour(
    worker: &Worker<impl Network>,
    owner: &Account,
    user: &Account,
    boost: &Contract,
) -> anyhow::Result<()> {

    let rewards = maplit::hashmap! {
        "token01".parse::<AccountId>().unwrap() => parse_near!("1 N").to_string(),
        "token02".parse::<AccountId>().unwrap() => parse_near!("2 N").to_string(),
    };

    Ok(())
}

/// test manage operators
async fn test_operators(
    worker: &Worker<impl Network>,
    owner: &Account,
    user: &Account,
    boost: &Contract,
) -> anyhow::Result<()> {
    let outcome = owner
        .call(&worker, boost.id(), "extend_operators")
        .args_json(json!({
            "operators": vec![user.id()],
        }))?
        .deposit(0)
        .transact()
        .await;
    assert!(outcome.is_err());
    if let Err(err) = outcome {
        assert!(err
            .to_string()
            .contains("Requires attached deposit of exactly 1 yoctoNEAR"));
    }

    let outcome = user
        .call(&worker, boost.id(), "extend_operators")
        .args_json(json!({
            "operators": vec![user.id()],
        }))?
        .deposit(1)
        .transact()
        .await;
    assert!(outcome.is_err());
    if let Err(err) = outcome {
        assert!(err
            .to_string()
            .contains("E002: not allowed for the caller"));
    }

    let outcome = owner
        .call(&worker, boost.id(), "remove_operators")
        .args_json(json!({
            "operators": vec![user.id()],
        }))?
        .deposit(0)
        .transact()
        .await;
    assert!(outcome.is_err());
    if let Err(err) = outcome {
        assert!(err
            .to_string()
            .contains("Requires attached deposit of exactly 1 yoctoNEAR"));
    }

    let outcome = user
        .call(&worker, boost.id(), "remove_operators")
        .args_json(json!({
            "operators": vec![user.id()],
        }))?
        .deposit(1)
        .transact()
        .await;
    assert!(outcome.is_err());
    if let Err(err) = outcome {
        assert!(err
            .to_string()
            .contains("E002: not allowed for the caller"));
    }

    let outcome = user
        .call(&worker, boost.id(), "create_seed")
        .args_json(json!({
            "seed_id": "1".to_string(),
            "seed_decimal": 18,
        }))?
        .deposit(1)
        .transact()
        .await;
    assert!(outcome.is_err());
    if let Err(err) = outcome {
        assert!(err
            .to_string()
            .contains("E002: not allowed for the caller"));
    }

    owner
        .call(&worker, boost.id(), "extend_operators")
        .args_json(json!({
            "operators": vec![user.id()],
        }))?
        .deposit(1)
        .transact()
        .await?;
    
    let md: Metadata = worker
        .view(boost.id(), "get_metadata", Vec::new())
        .await?
        .json::<Metadata>()?;
    assert_eq!(md.operators.get(0), Some(user.id()));

    user
        .call(&worker, boost.id(), "create_seed")
        .args_json(json!({
            "seed_id": "1".to_string(),
            "seed_decimal": 18,
        }))?
        .deposit(1)
        .transact()
        .await?;

    owner
        .call(&worker, boost.id(), "remove_operators")
        .args_json(json!({
            "operators": vec![user.id()],
        }))?
        .deposit(1)
        .transact()
        .await?;
    
    let md: Metadata = worker
        .view(boost.id(), "get_metadata", Vec::new())
        .await?
        .json::<Metadata>()?;
    assert_eq!(md.operators.len(), 0);

    Ok(())
}

/// test pause resume
async fn test_pause_resume(
    worker: &Worker<impl Network>,
    owner: &Account,
    user: &Account,
    boost: &Contract,
) -> anyhow::Result<()> {
    let outcome = owner
        .call(&worker, boost.id(), "pause_contract")
        .args_json(json!({}))?
        .deposit(0)
        .transact()
        .await;
    assert!(outcome.is_err());
    if let Err(err) = outcome {
        assert!(err
            .to_string()
            .contains("Requires attached deposit of exactly 1 yoctoNEAR"));
    }

    let outcome = user
        .call(&worker, boost.id(), "pause_contract")
        .args_json(json!({}))?
        .deposit(1)
        .transact()
        .await;
    assert!(outcome.is_err());
    if let Err(err) = outcome {
        assert!(err
            .to_string()
            .contains("E002: not allowed for the caller"));
    }

    owner
        .call(&worker, boost.id(), "pause_contract")
        .args_json(json!({
            "owner_id": user.id(),
        }))?
        .deposit(1)
        .transact()
        .await?;
    
    let md: Metadata = worker
        .view(boost.id(), "get_metadata", Vec::new())
        .await?
        .json::<Metadata>()?;
    assert_eq!(&md.state, &RunningState::Paused);

    let outcome = owner
        .call(&worker, boost.id(), "create_seed")
        .args_json(json!({
            "seed_id": "0".to_string(),
            "seed_decimal": 18,
        }))?
        .deposit(1)
        .transact()
        .await;
    assert!(outcome.is_err());
    if let Err(err) = outcome {
        assert!(err
            .to_string()
            .contains("E004: contract paused"));
    }

    owner
        .call(&worker, boost.id(), "resume_contract")
        .args_json(json!({
            "owner_id": user.id(),
        }))?
        .deposit(1)
        .transact()
        .await?;

    owner
        .call(&worker, boost.id(), "create_seed")
        .args_json(json!({
            "seed_id": "0".to_string(),
            "seed_decimal": 18,
        }))?
        .deposit(1)
        .transact()
        .await?;

    Ok(())
}

/// test ownership
async fn test_ownership(
    worker: &Worker<impl Network>,
    owner: &Account,
    user: &Account,
    boost: &Contract,
) -> anyhow::Result<()> {
    let outcome = owner
        .call(&worker, boost.id(), "set_owner")
        .args_json(json!({
            "owner_id": user.id(),
        }))?
        .deposit(0)
        .transact()
        .await;
    assert!(outcome.is_err());
    if let Err(err) = outcome {
        assert!(err
            .to_string()
            .contains("Requires attached deposit of exactly 1 yoctoNEAR"));
    }

    let outcome = user
        .call(&worker, boost.id(), "set_owner")
        .args_json(json!({
            "owner_id": user.id(),
        }))?
        .deposit(1)
        .transact()
        .await;
    assert!(outcome.is_err());
    if let Err(err) = outcome {
        assert!(err.to_string().contains("E002: not allowed for the caller"));
    }

    owner
        .call(&worker, boost.id(), "set_owner")
        .args_json(json!({
            "owner_id": user.id(),
        }))?
        .deposit(1)
        .transact()
        .await?;

    let md: Metadata = worker
        .view(boost.id(), "get_metadata", Vec::new())
        .await?
        .json::<Metadata>()?;
    assert_eq!(&md.owner_id, user.id());

    user
        .call(&worker, boost.id(), "set_owner")
        .args_json(json!({
            "owner_id": owner.id(),
        }))?
        .deposit(1)
        .transact()
        .await?;

    let md: Metadata = worker
        .view(boost.id(), "get_metadata", Vec::new())
        .await?
        .json::<Metadata>()?;
    assert_eq!(&md.owner_id, owner.id());

    

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let boost_wasm = std::fs::read(BOOST_FILEPATH)?;
    let boost = worker.dev_deploy(&boost_wasm).await?;

    let owner = worker.root_account();
    let user = worker.dev_create_account().await?;

    boost
        .call(&worker, "new")
        .args_json(json!({
                "owner_id": owner.id(),
        }))?
        .transact()
        .await?;
    
    owner
        .call(&worker, boost.id(), "resume_contract")
        .args_json(json!({
            "owner_id": user.id(),
        }))?
        .deposit(1)
        .transact()
        .await?;

    let md: Metadata = worker
        .view(boost.id(), "get_metadata", Vec::new())
        .await?
        .json::<Metadata>()?;
    assert_eq!(&md.owner_id, owner.id());
    assert_eq!(&md.state, &RunningState::Running);

    if let Err(err) = test_ownership(&worker, &owner, &user, &boost).await {
        println!("Test ownership Error: {}", err.to_string());
    } else {
        println!("Test ownership OK.");
    }

    if let Err(err) = test_pause_resume(&worker, &owner, &user, &boost).await {
        println!("Test pause_resume Error: {}", err.to_string());
    } else {
        println!("Test pause_resume OK.");
    }

    if let Err(err) = test_operators(&worker, &owner, &user, &boost).await {
        println!("Test operators Error: {}", err.to_string());
    } else {
        println!("Test operators OK.");
    }

    Ok(())
}
