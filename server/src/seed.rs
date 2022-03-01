use crate::change::api::{ChangeApi, SearchNonprofitsRequestBuilder};
use affect_storage::stores::nonprofit::{NewNonprofitRow, NonprofitStore, PgNonprofitStore};
use anyhow::Context;
use chrono::Utc;
use log::debug;
use std::sync::Arc;

// Seeds change table with some real data.
pub async fn insert_nonprofit(
    nonprofit_store: Arc<PgNonprofitStore>,
    api: Arc<ChangeApi>,
) -> Result<(), anyhow::Error> {
    let existing_nonprofits = nonprofit_store.list_nonprofits(10, None).await?;
    if existing_nonprofits.len() >= 1 {
        debug!("Nonprofits already seeded: {:?}", existing_nonprofits);
        return Ok(());
    }

    let change_nonprofit = api
        .search_nonprofits(
            &SearchNonprofitsRequestBuilder::default()
                .search_term("Watsi".to_string())
                .build()
                .context("search request failed")?,
        )
        .await?
        .nonprofits
        .remove(0);

    let now = Utc::now();
    let new_row = NewNonprofitRow {
        create_time: now,
        update_time: now,
        change_nonprofit_id: change_nonprofit.id,
        icon_url: change_nonprofit.icon_url,
        title: change_nonprofit.name,
        ein: change_nonprofit.ein,
        mission: change_nonprofit.mission,
        category: change_nonprofit.category,
    };

    debug!("Seeding nonprofit: {:?}", new_row);
    nonprofit_store.add_nonprofit(new_row).await?;

    Ok(())
}
