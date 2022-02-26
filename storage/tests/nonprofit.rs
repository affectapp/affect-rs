use affect_storage::stores::nonprofit::*;
use chrono::{TimeZone, Utc};
use std::sync::Arc;
use testcontainers::clients::Cli;
use uuid::Uuid;

mod common;

#[tokio::test]
async fn create_nonprofit() -> Result<(), Box<dyn std::error::Error>> {
    let docker_cli = Cli::default();
    let pg_container = common::setup_pg_container(&docker_cli).await?;
    let nonprofit_store = PgNonprofitStore::new(Arc::new(pg_container.pool));

    let mut expected_nonprofit = NonprofitRow {
        nonprofit_id: Uuid::new_v4(),
        create_time: Utc.timestamp(500, 0),
        update_time: Utc.timestamp(1000, 0),
        change_nonprofit_id: "test_nonprofit_id".to_string(),
        icon_url: "test_icon_url".to_string(),
        title: "title".to_string(),
        ein: "ein".to_string(),
        mission: "mission".to_string(),
        category: "category".to_string(),
    };

    // Insert nonprofit.
    let nonprofit = nonprofit_store
        .add_nonprofit(NewNonprofitRow {
            create_time: expected_nonprofit.create_time.clone(),
            update_time: expected_nonprofit.update_time.clone(),
            change_nonprofit_id: expected_nonprofit.change_nonprofit_id.clone(),
            icon_url: expected_nonprofit.icon_url.clone(),
            title: expected_nonprofit.title.clone(),
            ein: expected_nonprofit.ein.clone(),
            mission: expected_nonprofit.mission.clone(),
            category: expected_nonprofit.category.clone(),
        })
        .await?;

    // Id is generated on insert, don't test that.
    expected_nonprofit.nonprofit_id = nonprofit.nonprofit_id;

    assert_eq!(nonprofit, expected_nonprofit);
    Ok(())
}
