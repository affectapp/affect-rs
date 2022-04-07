use crate::protobuf::into::{IntoProto, ProtoInto};
use affect_api::affect::{
    affiliate_service_server::AffiliateService, Affiliate, AffiliateLink, AffiliateLinkType,
    BusinessType, CreateAffiliateRequest, GenerateAffiliateLinkRequest, RefreshAffiliateRequest,
};
use affect_status::{internal, invalid_argument, not_found, well_known::UnwrapField, Status};
use affect_storage::{
    database::client::DatabaseClient,
    database::store::{OnDemandStore, TransactionalStore},
    stores::affiliate::{AffiliateStore, NewAffiliateManagerRow, NewAffiliateRow},
};
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use std::marker::PhantomData;
use std::sync::Arc;
use tonic::{Request, Response};

use affect_storage::stores::affiliate::BusinessType as StoreBusinessType;

pub struct AffiliateServiceImpl<Db, Store, TStore> {
    database: Arc<Db>,
    stripe: Arc<stripe::Client>,
    _marker: PhantomData<(Store, TStore)>,
}

impl<Db, Store, TStore> AffiliateServiceImpl<Db, Store, TStore> {
    pub fn new(database: Arc<Db>, stripe: Arc<stripe::Client>) -> Self {
        Self {
            database,
            stripe,
            _marker: PhantomData,
        }
    }
}

#[async_trait]
impl<Db, Store, TStore> AffiliateService for AffiliateServiceImpl<Db, Store, TStore>
where
    Db: DatabaseClient<Store, TStore> + 'static,
    Store: AffiliateStore + OnDemandStore + 'static,
    TStore: AffiliateStore + TransactionalStore + 'static,
    Self: Sync + Send,
{
    async fn create_affiliate(
        &self,
        request: Request<CreateAffiliateRequest>,
    ) -> Result<Response<Affiliate>, Status> {
        let message = request.into_inner();

        let user_id = message
            .user_id
            .clone()
            .unwrap_field("user_id")?
            .proto_field_into("user_id")?;
        let company_name = message.company_name.clone().unwrap_field("company_name")?;
        let contact_email = message
            .contact_email
            .clone()
            .unwrap_field("contact_email")?;

        let asserted_nonprofit_id = message
            .asserted_nonprofit_id
            .clone()
            .unwrap_field("asserted_nonprofit_id")?
            .proto_field_into("asserted_nonprofit_id")?;

        let stripe_business_type = match message.business_type() {
            BusinessType::Unspecified => {
                return Err(invalid_argument!("'business_type' must be specified"))
            }
            BusinessType::Individual => stripe::AccountBusinessType::Individual,
            BusinessType::Company => stripe::AccountBusinessType::Company,
            BusinessType::Nonprofit => stripe::AccountBusinessType::NonProfit,
            BusinessType::GovernmentEntity => stripe::AccountBusinessType::GovernmentEntity,
        };

        let mut create_stripe_account = stripe::CreateAccount::new();
        create_stripe_account.type_ = Some(stripe::AccountType::Express);
        let mut capabilities = stripe::CreateAccountCapabilities::default();
        capabilities.transfers = Some(stripe::CreateAccountCapabilitiesTransfers {
            requested: Some(true),
        });
        capabilities.card_payments = Some(stripe::CreateAccountCapabilitiesCardPayments {
            // requested: Some(true),
            requested: Some(false),
        });
        create_stripe_account.capabilities = Some(capabilities);
        create_stripe_account.email = Some(&contact_email);
        create_stripe_account.business_type = Some(stripe_business_type);
        let mut company = stripe::CompanyParams::default();
        company.name = Some(company_name);
        create_stripe_account.company = Some(company);

        let stripe_account = stripe::Account::create(&self.stripe, create_stripe_account)
            .await
            .map_err(|e| internal!("failed to create stripe account: {:?}", e))?;

        let now = Utc::now();

        let txn = self.database.begin().await?;

        let affiliate_row = txn
            .add_affiliate(NewAffiliateRow {
                create_time: now,
                update_time: now,
                stripe_account_id: stripe_account.id.to_string(),
                company_name: stripe_account
                    .company
                    .map(|c| c.name)
                    .flatten()
                    .ok_or(internal!("expected stripe account company name"))?,
                contact_email: stripe_account
                    .email
                    .ok_or(internal!("expected stripe account email"))?,
                business_type: match stripe_account.business_type {
                    Some(stripe::AccountBusinessType::Individual) => StoreBusinessType::Individual,
                    Some(stripe::AccountBusinessType::Company) => StoreBusinessType::Company,
                    Some(stripe::AccountBusinessType::NonProfit) => StoreBusinessType::Nonprofit,
                    Some(stripe::AccountBusinessType::GovernmentEntity) => {
                        StoreBusinessType::GovernmentEntity
                    }
                    None => {
                        return Err(internal!("expected stripe account business type"));
                    }
                },
                asserted_nonprofit_id,
            })
            .await?;
        txn.add_affiliate_manager(NewAffiliateManagerRow {
            affiliate_id: affiliate_row.affiliate_id.clone(),
            user_id,
            create_time: now,
            update_time: now,
        })
        .await?;
        let affiliate_full_row = txn
            .find_affiliate_by_id(affiliate_row.affiliate_id.clone())
            .await?
            .ok_or(internal!("expected to find created affiliate"))?;
        txn.commit().await?;

        Ok(Response::new(affiliate_full_row.into_proto()?))
    }

    async fn generate_affiliate_link(
        &self,
        request: Request<GenerateAffiliateLinkRequest>,
    ) -> Result<Response<AffiliateLink>, Status> {
        let message = request.into_inner();
        let affiliate_id = message
            .affiliate_id
            .to_string()
            .unwrap_field("affiliate_id")?
            .proto_field_into("affiliate_id")?;

        let full_affiliate_row = match self
            .database
            .on_demand()
            .find_affiliate_by_id(affiliate_id)
            .await?
        {
            Some(row) => row,
            None => {
                return Err(not_found!("affiliate not found"));
            }
        };

        let stripe_account_id = full_affiliate_row
            .affiliate
            .stripe_account_id
            .parse::<stripe::AccountId>()
            .map_err(|e| internal!("failed to parse stripe account id: {:?}", e))?;

        let link = match message.link_type() {
            AffiliateLinkType::Unspecified => {
                return Err(invalid_argument!("'link_type' must be specified"));
            }
            AffiliateLinkType::Onboarding => {
                let account_link = stripe::AccountLink::create(
                    &self.stripe,
                    stripe::CreateAccountLink {
                        account: stripe_account_id,
                        type_: stripe::AccountLinkType::AccountOnboarding,
                        collect: None,
                        expand: &[],
                        refresh_url: Some(&format!(
                            "https://web.affect.app/#/affiliate/{}/stripe/onboarding",
                            full_affiliate_row.affiliate.affiliate_id.to_string()
                        )),
                        return_url: Some(&format!(
                            "https://web.affect.app/#/affiliate/{}/stripe/return",
                            full_affiliate_row.affiliate.affiliate_id.to_string(),
                        )),
                    },
                )
                .await
                .map_err(|e| internal!("failed to create account link: {:?}", e))?;

                let expire_time = Utc.timestamp(account_link.expires_at, 0);
                AffiliateLink {
                    url: account_link.url,
                    link_type: AffiliateLinkType::Onboarding as i32,
                    expire_time: Some(expire_time.into_proto()?),
                }
            }
            AffiliateLinkType::Login => {
                let login_link = stripe::LoginLink::create(
                    &self.stripe,
                    &stripe_account_id,
                    &format!(
                        "https://web.affect.app/#/affiliate/{}/stripe/return",
                        full_affiliate_row.affiliate.affiliate_id.to_string(),
                    ),
                )
                .await
                .map_err(|e| internal!("failed to create login link: {:?}", e))?;

                AffiliateLink {
                    url: login_link.url,
                    link_type: AffiliateLinkType::Login as i32,
                    expire_time: None,
                }
            }
        };

        Ok(Response::new(link))
    }

    async fn refresh_affiliate(
        &self,
        request: Request<RefreshAffiliateRequest>,
    ) -> Result<Response<Affiliate>, Status> {
        let message = request.into_inner();
        let affiliate_id = message
            .affiliate_id
            .unwrap_field("affiliate_id")?
            .proto_field_into("affiliate_id")?;

        let full_affiliate_row = match self
            .database
            .on_demand()
            .find_affiliate_by_id(affiliate_id)
            .await?
        {
            Some(row) => row,
            None => {
                return Err(not_found!("affiliate not found"));
            }
        };

        let stripe_account_id = full_affiliate_row
            .affiliate
            .stripe_account_id
            .parse::<stripe::AccountId>()
            .map_err(|e| internal!("failed to parse stripe account id: {:?}", e))?;

        let stripe_account = stripe::Account::retrieve(&self.stripe, &stripe_account_id, &[])
            .await
            .map_err(|e| internal!("failed to retrieve stripe account: {:?}", e))?;

        let _payouts_enabled = stripe_account
            .payouts_enabled
            .ok_or(internal!("expected stripe account 'payouts_enabled' field"))?;
        let _country = stripe_account.country;
        let _business_name = stripe_account.business_profile.map(|p| p.name).flatten();

        Ok(Response::new(full_affiliate_row.into_proto()?))
    }
}
