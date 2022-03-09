use crate::prost::from::ProtoFrom;
use affect_api::affect::{Account, Item};
use affect_storage::stores::account::AccountRow;
use affect_storage::stores::item::ItemRow;
use prost_types::Timestamp;
use tonic::Status;

impl ProtoFrom<(ItemRow, Vec<AccountRow>)> for Item {
    fn proto_from(value: (ItemRow, Vec<AccountRow>)) -> Result<Self, Status> {
        let (item_row, account_rows) = value;
        Ok(Item {
            item_id: item_row.item_id.to_string(),
            create_time: Some(Timestamp::proto_from(item_row.create_time)?),
            update_time: Some(Timestamp::proto_from(item_row.update_time)?),
            user_id: item_row.user_id.to_string(),
            accounts: account_rows
                .into_iter()
                .map(|account_row| {
                    Ok(Account {
                        account_id: account_row.account_id.to_string(),
                        create_time: Some(Timestamp::proto_from(account_row.create_time)?),
                        update_time: Some(Timestamp::proto_from(account_row.update_time)?),
                        item_id: account_row.item_id.to_string(),
                        name: account_row.name,
                        mask: account_row.mask.unwrap_or("".to_string()),
                    })
                })
                .collect::<Result<Vec<Account>, Status>>()?,
        })
    }
}
