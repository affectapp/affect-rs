use affect_api::affect::{
    donation_service_server::DonationService, CreateOneoffDonationRequest, Donation, *,
};
use affect_storage::database::{
    client::DatabaseClient,
    store::{OnDemandStore, TransactionalStore},
};
use async_trait::async_trait;
use std::{marker::PhantomData, sync::Arc};
use tonic::{Request, Response, Status};

pub struct DonationServiceImpl<Db, Store, TStore> {
    database: Arc<Db>,
    _marker: PhantomData<(Store, TStore)>,
}

impl<Db, Store, TStore> DonationServiceImpl<Db, Store, TStore> {
    pub fn new(database: Arc<Db>) -> Self {
        Self {
            database,
            _marker: PhantomData,
        }
    }
}

#[async_trait]
impl<Db, Store, TStore> DonationService for DonationServiceImpl<Db, Store, TStore>
where
    Db: DatabaseClient<Store, TStore> + 'static,
    Store: OnDemandStore + 'static,
    TStore: TransactionalStore + 'static,
    Self: Sync + Send,
{
    async fn create_oneoff_donation(
        &self,
        request: Request<CreateOneoffDonationRequest>,
    ) -> Result<Response<Donation>, Status> {
        todo!()
    }

    async fn get_donation(
        &self,
        request: Request<GetDonationRequest>,
    ) -> Result<Response<Donation>, Status> {
        todo!()
    }
}
