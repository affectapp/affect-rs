use crate::protobuf::{from::ProtoFrom, into::IntoProto};
use affect_api::affect::Nonprofit;
use affect_storage::stores::nonprofit::FullNonprofitRow;
use tonic::Status;

impl ProtoFrom<FullNonprofitRow> for Nonprofit {
    fn proto_from(value: FullNonprofitRow) -> Result<Self, Status> {
        let nonprofit_row = value.nonprofit;
        let affiliate_row = value.affiliate;

        Ok(Nonprofit {
            nonprofit_id: nonprofit_row.nonprofit_id.to_string(),
            create_time: Some(nonprofit_row.create_time.into_proto()?),
            update_time: Some(nonprofit_row.update_time.into_proto()?),
            icon_url: nonprofit_row.icon_url,
            name: nonprofit_row.name,
            ein: nonprofit_row.ein,
            mission: nonprofit_row.mission,
            category: nonprofit_row.category,
            affiliate_id: match affiliate_row {
                Some(affiliate_row) => affiliate_row.affiliate_id.to_string(),
                None => "".to_string(),
            },
        })
    }
}
