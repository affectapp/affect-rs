use crate::protobuf::from::ProtoFrom;
use affect_api::affect::Nonprofit;
use affect_storage::stores::nonprofit::NonprofitRow;
use prost_types::Timestamp;
use tonic::Status;

impl ProtoFrom<NonprofitRow> for Nonprofit {
    fn proto_from(value: NonprofitRow) -> Result<Self, Status> {
        Ok(Nonprofit {
            nonprofit_id: value.nonprofit_id.to_string(),
            create_time: Some(Timestamp::proto_from(value.create_time)?),
            update_time: Some(Timestamp::proto_from(value.update_time)?),
            icon_url: value.icon_url,
            name: value.name,
            ein: value.ein,
            mission: value.mission,
            category: value.category,
        })
    }
}
