use crate::change::client::{ChangeClient, SearchNonprofitsRequestBuilder};
use affect_storage::stores::nonprofit::{NewNonprofitRow, NonprofitStore, PgNonprofitStore};
use anyhow::Context;
use chrono::Utc;
use log::debug;
use std::sync::Arc;

// Seeds change table with some real data.
pub async fn insert_nonprofits(
    nonprofit_store: Arc<PgNonprofitStore>,
    client: Arc<ChangeClient>,
) -> Result<(), anyhow::Error> {
    let existing_nonprofits = nonprofit_store.list_nonprofits(20, None).await?;
    if existing_nonprofits.len() >= 10 {
        debug!(
            "Num nonprofits already seeded: {:?}",
            existing_nonprofits.len()
        );
        return Ok(());
    }

    let change_nonprofits = client
        .search_nonprofits(
            &SearchNonprofitsRequestBuilder::default()
                .build()
                .context("search request failed")?,
        )
        .await?
        .nonprofits;

    for change_nonprofit in change_nonprofits {
        let now = Utc::now();
        let new_row = NewNonprofitRow {
            create_time: now,
            update_time: now,
            change_nonprofit_id: change_nonprofit.id,
            icon_url: change_nonprofit.icon_url,
            name: change_nonprofit.name,
            ein: change_nonprofit.ein,
            mission: change_nonprofit.mission,
            category: change_nonprofit.category,
        };

        debug!("Seeding nonprofit: {:?}", new_row);
        nonprofit_store.add_nonprofit(new_row).await?;
    }
    Ok(())
}
