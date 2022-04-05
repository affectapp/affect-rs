use crate::protobuf::from::ProtoFrom;
use affect_api::affect::{Cause, CauseRecipient};
use affect_storage::stores::cause::{CauseRecipientRow, CauseRow, FullCauseRow};
use prost_types::Timestamp;
use tonic::Status;

impl ProtoFrom<(CauseRow, Vec<CauseRecipientRow>)> for Cause {
    fn proto_from(value: (CauseRow, Vec<CauseRecipientRow>)) -> Result<Self, Status> {
        let (cause_row, cause_recipient_rows) = value;
        Ok(Cause {
            cause_id: cause_row.cause_id.to_string(),
            create_time: Some(Timestamp::proto_from(cause_row.create_time)?),
            update_time: Some(Timestamp::proto_from(cause_row.update_time)?),
            user_id: cause_row.user_id.to_string(),
            recipients: cause_recipient_rows
                .into_iter()
                .map(|cause_recipient_row| CauseRecipient {
                    cause_id: cause_recipient_row.cause_id.to_string(),
                    nonprofit_id: cause_recipient_row.nonprofit_id.to_string(),
                })
                .collect(),
        })
    }
}

impl ProtoFrom<FullCauseRow> for Cause {
    fn proto_from(value: FullCauseRow) -> Result<Self, Status> {
        Ok(Cause {
            cause_id: value.cause.cause_id.to_string(),
            create_time: Some(Timestamp::proto_from(value.cause.create_time)?),
            update_time: Some(Timestamp::proto_from(value.cause.update_time)?),
            user_id: value.cause.user_id.to_string(),
            recipients: value
                .cause_recipients
                .inner()
                .into_iter()
                .map(|cause_recipient_row| CauseRecipient {
                    cause_id: cause_recipient_row.cause_id.to_string(),
                    nonprofit_id: cause_recipient_row.nonprofit_id.to_string(),
                })
                .collect(),
        })
    }
}
