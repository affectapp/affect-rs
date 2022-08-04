use crate::{
    money::Money,
    protobuf::from::{FromProto, ProtoFrom},
};
use affect_status::invalid_argument;
use chrono::{DateTime, TimeZone, Utc};
use iso_currency::Currency;
use prost_types::Timestamp;
use tonic::Status;
use uuid::Uuid;

use affect_api::google::r#type::Money as MoneyProto;

impl FromProto<String> for Uuid {
    fn from_proto(proto: String) -> Result<Self, Status> {
        Ok(Uuid::parse_str(&proto)
            .map_err(|e| invalid_argument!("could not parse as uuid: {:?}", e))?)
    }
}

impl ProtoFrom<Uuid> for String {
    fn proto_from(value: Uuid) -> Result<Self, Status> {
        Ok(value.to_string())
    }
}

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

impl FromProto<MoneyProto> for Money {
    fn from_proto(proto: MoneyProto) -> Result<Self, Status> {
        let currency = Currency::from_code(&proto.currency_code).ok_or(invalid_argument!(
            "invalid currency: {0}",
            proto.currency_code
        ))?;
        Ok(Money {
            currency,
            units: proto.units,
            nanos: proto.nanos,
        })
    }
}

impl ProtoFrom<Money> for MoneyProto {
    fn proto_from(value: Money) -> Result<Self, Status> {
        Ok(MoneyProto {
            currency_code: value.currency.code().to_string(),
            units: value.units,
            nanos: value.nanos,
        })
    }
}
