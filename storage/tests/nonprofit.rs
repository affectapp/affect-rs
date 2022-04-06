use affect_storage::{database::client::DatabaseClient, stores::nonprofit::*};
use chrono::{TimeZone, Utc};
use testcontainers::clients::Cli;
use uuid::Uuid;

mod common;

#[tokio::test]
async fn create_nonprofit() -> Result<(), anyhow::Error> {
    let docker = Cli::default();
    let container = common::setup_pg_container(&docker).await?;
    let store = container.pool.on_demand();

    let mut expected_nonprofit = NonprofitRow {
        nonprofit_id: Uuid::new_v4(),
        create_time: Utc.timestamp(500, 0),
        update_time: Utc.timestamp(1000, 0),
        change_nonprofit_id: Some("test_nonprofit_id".to_string()),
        icon_url: "test_icon_url".to_string(),
        name: "name".to_string(),
        ein: "ein".to_string(),
        mission: "mission".to_string(),
        category: "category".to_string(),
        affiliate_id: None,
    };

    // Insert nonprofit.
    let nonprofit = store
        .add_nonprofit(NewNonprofitRow {
            create_time: expected_nonprofit.create_time.clone(),
            update_time: expected_nonprofit.update_time.clone(),
            change_nonprofit_id: expected_nonprofit.change_nonprofit_id.clone(),
            icon_url: expected_nonprofit.icon_url.clone(),
            name: expected_nonprofit.name.clone(),
            ein: expected_nonprofit.ein.clone(),
            mission: expected_nonprofit.mission.clone(),
            category: expected_nonprofit.category.clone(),
            affiliate_id: None,
        })
        .await?;

    // Id is generated on insert, don't test that.
    expected_nonprofit.nonprofit_id = nonprofit.nonprofit_id;

    assert_eq!(nonprofit, expected_nonprofit);
    Ok(())
}

#[tokio::test]
async fn find_nonprofit_by_id_exists() -> Result<(), anyhow::Error> {
    let docker = Cli::default();
    let container = common::setup_pg_container(&docker).await?;
    let store = container.pool.on_demand();

    // Seed database with nonprofit
    let mut expected_nonprofit = NonprofitRow {
        nonprofit_id: Uuid::new_v4(),
        create_time: Utc.timestamp(500, 0),
        update_time: Utc.timestamp(1000, 0),
        change_nonprofit_id: Some("test_nonprofit_id".to_string()),
        icon_url: "test_icon_url".to_string(),
        name: "name".to_string(),
        ein: "ein".to_string(),
        mission: "mission".to_string(),
        category: "category".to_string(),
        affiliate_id: None,
    };
    let inserted_nonprofit = store
        .add_nonprofit(NewNonprofitRow {
            create_time: expected_nonprofit.create_time.clone(),
            update_time: expected_nonprofit.update_time.clone(),
            change_nonprofit_id: expected_nonprofit.change_nonprofit_id.clone(),
            icon_url: expected_nonprofit.icon_url.clone(),
            name: expected_nonprofit.name.clone(),
            ein: expected_nonprofit.ein.clone(),
            mission: expected_nonprofit.mission.clone(),
            category: expected_nonprofit.category.clone(),
            affiliate_id: None,
        })
        .await?;
    expected_nonprofit.nonprofit_id = inserted_nonprofit.nonprofit_id;

    let full_nonprofit = store
        .find_nonprofit_by_id(inserted_nonprofit.nonprofit_id)
        .await?;

    assert_eq!(
        full_nonprofit,
        Some(FullNonprofitRow {
            nonprofit: expected_nonprofit,
            affiliate: None,
        })
    );
    Ok(())
}

#[tokio::test]
async fn find_nonprofit_by_id_none() -> Result<(), anyhow::Error> {
    let docker = Cli::default();
    let container = common::setup_pg_container(&docker).await?;
    let store = container.pool.on_demand();

    assert_eq!(store.find_nonprofit_by_id(Uuid::new_v4()).await?, None);
    Ok(())
}
