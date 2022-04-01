use crate::protobuf::{from::ProtoFrom, into::IntoProto};
use affect_api::affect::{Affiliate, AffiliateManager};
use affect_storage::stores::affiliate::AffiliateFullRow;
use tonic::Status;

// Same type name.
use affect_api::affect::BusinessType as ProtoBusinessType;
use affect_storage::stores::affiliate::BusinessType as StoreBusinessType;

impl ProtoFrom<AffiliateFullRow> for Affiliate {
    fn proto_from(affiliate_row: AffiliateFullRow) -> Result<Self, Status> {
        let business_type: ProtoBusinessType = affiliate_row.business_type.into_proto()?;
        Ok(Affiliate {
            affiliate_id: affiliate_row.affiliate_id.into_proto()?,
            create_time: Some(affiliate_row.create_time.into_proto()?),
            update_time: Some(affiliate_row.update_time.into_proto()?),
            company_name: affiliate_row.company_name,
            contact_email: affiliate_row.contact_email,
            business_type: business_type as i32,
            managers: affiliate_row
                .affiliate_managers
                .inner()
                .into_iter()
                .map(|affiliate_manager_row| {
                    return AffiliateManager {
                        user_id: affiliate_manager_row.user_id.to_string(),
                    };
                })
                .collect(),
            asserted_nonprofit_id: affiliate_row.asserted_nonprofit_id.into_proto()?,
        })
    }
}

impl ProtoFrom<StoreBusinessType> for ProtoBusinessType {
    fn proto_from(value: affect_storage::stores::affiliate::BusinessType) -> Result<Self, Status> {
        match value {
            StoreBusinessType::Individual => Ok(Self::Individual),
            StoreBusinessType::Company => Ok(Self::Company),
            StoreBusinessType::Nonprofit => Ok(Self::Nonprofit),
            StoreBusinessType::GovernmentEntity => Ok(Self::GovernmentEntity),
        }
    }
}
