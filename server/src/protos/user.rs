use crate::prost::from::ProtoFrom;
use affect_api::affect::User;
use affect_storage::stores::user::UserRow;
use prost_types::Timestamp;
use tonic::Status;

impl ProtoFrom<UserRow> for User {
    fn proto_from(value: UserRow) -> Result<Self, Status> {
        Ok(User {
            user_id: value.user_id.to_string(),
            create_time: Some(Timestamp::proto_from(value.create_time)?),
            update_time: Some(Timestamp::proto_from(value.update_time)?),
            firebase_uid: value.firebase_uid.to_string(),
        })
    }
}
