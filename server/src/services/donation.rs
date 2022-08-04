use affect_api::affect::{
    donation_service_server::DonationService, CreateDonationRequest, Donation, *,
};
use affect_status::{
    internal, invalid_argument,
    well_known::{entity_not_found, UnwrapField},
};
use affect_storage::{
    database::{
        client::DatabaseClient,
        store::{OnDemandStore, TransactionalStore},
    },
    stores::{nonprofit::NonprofitStore, user::UserStore},
};
use async_trait::async_trait;
use log::info;
use std::{marker::PhantomData, sync::Arc};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::{money::Money, protobuf::into::ProtoInto};

pub struct DonationServiceImpl<Db, Store, TStore> {
    database: Arc<Db>,
    stripe: Arc<stripe::Client>,
    _marker: PhantomData<(Store, TStore)>,
}

impl<Db, Store, TStore> DonationServiceImpl<Db, Store, TStore> {
    pub fn new(database: Arc<Db>, stripe: Arc<stripe::Client>) -> Self {
        Self {
            database,
            stripe,
            _marker: PhantomData,
        }
    }
}

#[async_trait]
impl<Db, Store, TStore> DonationService for DonationServiceImpl<Db, Store, TStore>
where
    Db: DatabaseClient<Store, TStore> + 'static,
    Store: NonprofitStore + UserStore + OnDemandStore + 'static,
    TStore: TransactionalStore + 'static,
    Self: Sync + Send,
{
    async fn create_donation(
        &self,
        request: Request<CreateDonationRequest>,
    ) -> Result<Response<Donation>, Status> {
        let message = request.into_inner();
        let nonprofit_id: Uuid = message
            .nonprofit_id
            .clone()
            .unwrap_field("nonprofit_id")?
            .proto_field_into("nonprofit_id")?;
        let user_id: Uuid = message
            .user_id
            .clone()
            .unwrap_field("user_id")?
            .proto_field_into("user_id")?;
        let amount: Money = message
            .amount
            .clone()
            .unwrap_field("amount")?
            .proto_field_into("amount")?;
        let stripe_currency = amount
            .stripe_currency()
            .map_err(|e| invalid_argument!("failed to parse currency: {:?}", e))?;

        let user = self
            .database
            .on_demand()
            .find_user_by_id(user_id)
            .await?
            .ok_or(entity_not_found("user"))?;
        let customer_id = user
            .stripe_customer_id
            .parse()
            .map_err(|e| internal!("failed to parse stripe customer id: {:?}", e))?;

        let nonprofit = self
            .database
            .on_demand()
            .find_nonprofit_by_id(nonprofit_id)
            .await?
            .ok_or(entity_not_found("nonprofit"))?;
        let account_id = nonprofit
            .affiliate
            .ok_or(invalid_argument!("nonprofit is not affiliated"))?
            .stripe_account_id
            .parse()
            .map_err(|e| internal!("failed to parse stripe account id: {:?}", e))?;

        let mut create_token = stripe::CreateToken::default();
        create_token.customer = Some(customer_id);

        let nonprofit_stripe_client = (*self.stripe).clone().with_stripe_account(account_id);
        let stripe_token = stripe::Token::create(&nonprofit_stripe_client, create_token)
            .await
            .map_err(|e| internal!("failed to create stripe token: {:?}", e))?;

        let mut create_charge = stripe::CreateCharge::default();
        create_charge.amount = Some(amount.subunits_truncated());
        create_charge.currency = Some(stripe_currency);
        create_charge.source = Some(stripe::ChargeSourceParams::Token(stripe_token.id));
        let charge = stripe::Charge::create(&nonprofit_stripe_client, create_charge)
            .await
            .map_err(|e| internal!("failed to create stripe charge: {:?}", e))?;

        info!("Created charge: {:?}", charge);

        // - Insert DonationRow to donations table.
        // Ok(Response::new(Donation {
        //     donation_id: "".to_string(),
        //     create_time: None,
        //     update_time: None,
        //     nonprofit_id: nonprofit_id.into_proto()?,
        //     user_id: user_id.into_proto()?,
        //     amount: Some(amount.into_proto()?),
        //     cause_id: "".to_string(),
        //     status: None,
        // }))
        todo!()
    }

    async fn get_donation(
        &self,
        _request: Request<GetDonationRequest>,
    ) -> Result<Response<Donation>, Status> {
        todo!()
    }
}
