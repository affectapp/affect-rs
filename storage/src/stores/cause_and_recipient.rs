use crate::{
    stores::{
        cause::{CauseRow, CauseStore, NewCauseRow},
        cause_recipient::{CauseRecipientRow, CauseRecipientStore, NewCauseRecipientRow},
    },
    Error, TransactionalStore,
};
use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

#[async_trait]
pub trait CauseAndRecipientStore: Send + Sync {
    async fn add_cause_and_recipients(
        &self,
        user_id: Uuid,
        recipient_nonprofit_ids: Vec<Uuid>,
    ) -> Result<(CauseRow, Vec<CauseRecipientRow>), Error>;
}

// Implement it for types that are cause stores, cause recipient stores, and are
// transactional, ensuring the operations are committed.
#[async_trait]
impl<S> CauseAndRecipientStore for S
where
    S: CauseStore + CauseRecipientStore + TransactionalStore,
{
    async fn add_cause_and_recipients(
        &self,
        user_id: Uuid,
        recipient_nonprofit_ids: Vec<Uuid>,
    ) -> Result<(CauseRow, Vec<CauseRecipientRow>), Error> {
        let now = Utc::now();
        let cause_row = self
            .add_cause(NewCauseRow {
                create_time: now,
                update_time: now,
                user_id,
                name: "some name".to_string(),
            })
            .await?;

        let mut recipient_rows = Vec::new();
        for recipient_nonprofit_id in recipient_nonprofit_ids {
            recipient_rows.push(
                self.add_cause_recipient(NewCauseRecipientRow {
                    cause_id: cause_row.cause_id.clone(),
                    nonprofit_id: recipient_nonprofit_id.clone(),
                    create_time: now,
                    update_time: now,
                })
                .await?,
            );
        }
        Ok((cause_row, recipient_rows))
    }
}
