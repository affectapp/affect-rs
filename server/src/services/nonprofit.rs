use crate::protobuf::into::IntoProto;
use affect_api::affect::{
    list_nonprofits_request::Filter, nonprofit_service_server::NonprofitService,
    ListNonprofitsRequest, *,
};
use affect_status::{invalid_argument, not_found};
use affect_storage::{
    database::{
        client::DatabaseClient,
        store::{OnDemandStore, TransactionalStore},
    },
    page_token::{PageToken, PageTokenable},
    stores::{
        affiliate::AffiliateStore,
        nonprofit::{NonprofitPageToken, NonprofitStore},
    },
};
use async_trait::async_trait;
use std::{
    cmp::{max, min},
    marker::PhantomData,
    sync::Arc,
};
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct NonprofitServiceImpl<Db, Store, TStore> {
    database: Arc<Db>,
    _marker: PhantomData<(Store, TStore)>,
}

impl<Db, Store, TStore> NonprofitServiceImpl<Db, Store, TStore> {
    pub fn new(database: Arc<Db>) -> Self {
        Self {
            database,
            _marker: PhantomData,
        }
    }
}

#[async_trait]
impl<Db, Store, TStore> NonprofitService for NonprofitServiceImpl<Db, Store, TStore>
where
    Db: DatabaseClient<Store, TStore> + 'static,
    Store: NonprofitStore + AffiliateStore + OnDemandStore + 'static,
    TStore: TransactionalStore + 'static,
    Self: Sync + Send,
{
    async fn get_nonprofit(
        &self,
        request: Request<GetNonprofitRequest>,
    ) -> Result<Response<Nonprofit>, Status> {
        let message = request.into_inner();

        let nonprofit_id = message
            .nonprofit_id
            .parse::<Uuid>()
            .map_err(|e| invalid_argument!("'nonprofit_id' is invalid: {:?}", e))?;

        let full_nonprofit_row = self
            .database
            .on_demand()
            .find_nonprofit_by_id(nonprofit_id)
            .await?
            .ok_or(not_found!("nonprofit not found"))?;

        Ok(Response::new(full_nonprofit_row.into_proto()?))
    }

    async fn list_nonprofits(
        &self,
        request: Request<ListNonprofitsRequest>,
    ) -> Result<Response<ListNonprofitsResponse>, Status> {
        let message = request.into_inner();

        let page_size = min(max(message.page_size, 10), 100);
        let limit: i64 = (page_size + 1).into();
        let page_token = NonprofitPageToken::deserialize_page_token(&message.page_token)
            .map_err(|e| invalid_argument!("'page_token' is invalid: {:?}", e))?;

        let (rows_plus_one, total_count) = match message.filter {
            Some(Filter::FilterBySearch(filter_by_search)) => {
                self.database
                    .on_demand()
                    .list_and_count_nonprofits_by_search(limit, page_token, &filter_by_search.query)
                    .await?
            }
            None => {
                self.database
                    .on_demand()
                    .list_and_count_nonprofits(limit, page_token)
                    .await?
            }
        };

        let (page_rows, next_page_rows) =
            rows_plus_one.split_at(min(rows_plus_one.len(), page_size as usize));

        // Map rows to protos and serialize page token.
        let mut nonprofits: Vec<Nonprofit> = Vec::new();
        for row in page_rows {
            nonprofits.push(row.clone().into_proto()?);
        }

        // Next page token or empty string.
        let next_page_token = next_page_rows
            .first()
            .map(|next_row| next_row.page_token().serialize_page_token())
            .unwrap_or(Ok("".to_string()))?;

        Ok(Response::new(ListNonprofitsResponse {
            nonprofits,
            next_page_token,
            total_count,
        }))
    }
}
