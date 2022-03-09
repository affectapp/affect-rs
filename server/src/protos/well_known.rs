use crate::protobuf::from::{FromProto, ProtoFrom};
use affect_status::invalid_argument;
use chrono::{DateTime, TimeZone, Utc};
use prost_types::Timestamp;
use tonic::Status;
use uuid::Uuid;

impl FromProto<String> for Uuid {
    fn from_proto(proto: String) -> Result<Self, Status> {
        Ok(Uuid::parse_str(&proto)
            .map_err(|e| invalid_argument!("could not parse uuid: {:?}", e))?)
    }
}

impl ProtoFrom<Uuid> for String {
    fn proto_from(value: Uuid) -> Result<Self, Status> {
        Ok(value.to_string())
    }
}

/// google.protobuf.Timestamp -> DateTime<Utc>
impl FromProto<Timestamp> for DateTime<Utc> {
    fn from_proto(proto: Timestamp) -> Result<Self, Status> {
        Ok(Utc.timestamp(
            proto.seconds,
            proto
                .nanos
                .try_into()
                .map_err(|e| invalid_argument!("invalid nanos: {:?}", e))?,
        ))
    }
}

/// DateTime<Utc> -> google.protobuf.Timestamp
impl ProtoFrom<DateTime<Utc>> for Timestamp {
    fn proto_from(value: DateTime<Utc>) -> Result<Self, Status> {
        Ok(Timestamp {
            seconds: value.timestamp(),
            nanos: value
                .timestamp_subsec_nanos()
                .try_into()
                .map_err(|e| invalid_argument!("invalid nanos: {:?}", e))?,
        })
    }
}
