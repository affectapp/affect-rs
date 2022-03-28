use crate::protobuf::{from::ProtoFrom, into::IntoProto};
use affect_api::affect::Affiliate;
use affect_storage::stores::affiliate::AffiliateRow;
use tonic::Status;

// Same type name.
use affect_api::affect::BusinessType as ProtoBusinessType;
use affect_storage::stores::affiliate::BusinessType as StoreBusinessType;

impl ProtoFrom<AffiliateRow> for Affiliate {
    fn proto_from(value: AffiliateRow) -> Result<Self, Status> {
        let business_type: ProtoBusinessType = value.business_type.into_proto()?;
        Ok(Affiliate {
            affiliate_id: value.affiliate_id.into_proto()?,
            create_time: Some(value.create_time.into_proto()?),
            update_time: Some(value.update_time.into_proto()?),
            company_name: value.company_name,
            contact_email: value.contact_email,
            business_type: business_type as i32,
        })
    }
}

impl ProtoFrom<StoreBusinessType> for ProtoBusinessType {
    fn proto_from(value: affect_storage::stores::affiliate::BusinessType) -> Result<Self, Status> {
        match value {
            affect_storage::stores::affiliate::BusinessType::Individual => Ok(Self::Individual),
            affect_storage::stores::affiliate::BusinessType::Company => Ok(Self::Company),
            affect_storage::stores::affiliate::BusinessType::Nonprofit => Ok(Self::Nonprofit),
            affect_storage::stores::affiliate::BusinessType::GovernmentEntity => {
                Ok(Self::GovernmentEntity)
            }
        }
    }
}
