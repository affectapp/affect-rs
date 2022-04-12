use crate::protobuf::{from::ProtoFrom, into::IntoProto};
use affect_api::affect::{Affiliate, AffiliateManager};
use affect_storage::models::affiliate::FullAffiliateRow;
use tonic::Status;

// Same type name.
use affect_api::affect::BusinessType as ProtoBusinessType;
use affect_storage::models::affiliate::BusinessType as StoreBusinessType;

impl ProtoFrom<FullAffiliateRow> for Affiliate {
    fn proto_from(value: FullAffiliateRow) -> Result<Self, Status> {
        let business_type: ProtoBusinessType = value.affiliate.business_type.into_proto()?;
        Ok(Affiliate {
            affiliate_id: value.affiliate.affiliate_id.into_proto()?,
            create_time: Some(value.affiliate.create_time.into_proto()?),
            update_time: Some(value.affiliate.update_time.into_proto()?),
            company_name: value.affiliate.company_name,
            contact_email: value.affiliate.contact_email,
            business_type: business_type as i32,
            managers: value
                .affiliate_managers
                .inner()
                .into_iter()
                .map(|affiliate_manager_row| {
                    return AffiliateManager {
                        user_id: affiliate_manager_row.user_id.to_string(),
                    };
                })
                .collect(),
            asserted_nonprofit_id: value.affiliate.asserted_nonprofit_id.into_proto()?,
        })
    }
}

impl ProtoFrom<StoreBusinessType> for ProtoBusinessType {
    fn proto_from(value: affect_storage::models::affiliate::BusinessType) -> Result<Self, Status> {
        match value {
            StoreBusinessType::Individual => Ok(Self::Individual),
            StoreBusinessType::Company => Ok(Self::Company),
            StoreBusinessType::Nonprofit => Ok(Self::Nonprofit),
            StoreBusinessType::GovernmentEntity => Ok(Self::GovernmentEntity),
        }
    }
}
