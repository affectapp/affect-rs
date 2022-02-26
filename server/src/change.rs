use derive_builder::Builder;
use hyper::StatusCode;
use reqwest::{Client, RequestBuilder, Response};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("request error: {0:?}")]
    Http(#[from] reqwest::Error),

    #[error("json deserialization error: {0:?}")]
    Json(#[from] serde_json::Error),

    #[error("change client error: status={status}, code={code}, title={title}")]
    ClientError {
        status: StatusCode,
        code: String,
        title: String,
    },
}

pub struct ChangeCredentials {
    public_key: String,
    secret_key: String,
}

impl ChangeCredentials {
    pub fn new(public_key: String, secret_key: String) -> Self {
        ChangeCredentials {
            public_key,
            secret_key,
        }
    }
}

pub struct ChangeApi {
    client: Client,
    creds: ChangeCredentials,
}

impl ChangeApi {
    pub fn new(creds: ChangeCredentials) -> Self {
        ChangeApi {
            client: Client::new(),
            creds,
        }
    }

    // See https://docs.getchange.io/api/#Nonprofits-Search-a-nonprofit
    pub async fn search_nonprofits(
        &self,
        request: &SearchNonprofitsRequest,
    ) -> Result<SearchNonprofitsResponse, Error> {
        let mut query_params: Vec<(&str, String)> = Vec::new();
        query_params.push(("public_key", self.creds.public_key.to_string()));
        if let Some(page) = &request.page {
            query_params.push(("page", page.to_string()));
        }
        if let Some(search_term) = &request.search_term {
            query_params.push(("search_term", search_term.to_string()));
        }
        if let Some(categories) = &request.categories {
            for category in categories {
                query_params.push(("categories[]", category.to_string()));
            }
        }
        Ok(self.get("nonprofits", Some(&query_params)).await?)
    }

    // See https://docs.getchange.io/api/#Donations-Create-a-donation
    pub async fn create_donation(&self, request: CreateDonationRequest) -> Result<Donation, Error> {
        Ok(self.post("donations", Some(&request)).await?)
    }

    // See https://docs.getchange.io/api/#Donations-List-your-donations
    pub async fn list_donations(
        &self,
        request: &ListDonationsRequest,
    ) -> Result<ListDonationsResponse, Error> {
        let mut query_params: HashMap<&str, String> = HashMap::new();
        query_params.insert("page", request.page.to_string());
        if let Some(account_id) = &request.account_id {
            query_params.insert("account_id", account_id.to_string());
        }
        Ok(self.get("donations", Some(&query_params)).await?)
    }

    // See https://docs.getchange.io/api/#Donations-Retrieve-a-donation
    pub async fn get_donation(&self, id: String) -> Result<Donation, Error> {
        Ok(self
            .get::<(), Donation>(&format!("donations/{0}", id), None)
            .await?)
    }

    // See https://docs.getchange.io/api/#Marketplace-Create-a-managed-account
    pub async fn create_account(&self, request: CreateAccountRequest) -> Result<Account, Error> {
        Ok(self.post("accounts", Some(&request)).await?)
    }

    // See https://docs.getchange.io/api/#Marketplace-Create-a-link-bank-token
    pub async fn create_link_bank_token(&self, id: String) -> Result<String, Error> {
        Ok(self
            .post_text_resp::<()>(&format!("accounts/{0}/link_bank_token", id), None)
            .await?)
    }

    // See https://docs.getchange.io/api/#Marketplace-Attach-a-bank-to-a-managed-account
    pub async fn attach_bank_account(
        &self,
        request: AttachBankAccountRequest,
    ) -> Result<Account, Error> {
        Ok(self
            .post("accounts/attach_bank_account", Some(&request))
            .await?)
    }

    async fn get<T, Resp>(&self, endpoint: &str, query_params: Option<&T>) -> Result<Resp, Error>
    where
        Resp: DeserializeOwned,
        T: Serialize + ?Sized,
    {
        let mut req = self
            .client
            .get(format!("https://api.getchange.io/api/v1/{0}", endpoint))
            .basic_auth(
                self.creds.public_key.to_string(),
                Some(self.creds.secret_key.to_string()),
            );
        if let Some(query_params) = query_params {
            req = req.query(query_params)
        }
        let response = self.send(req).await?;
        Ok(response.json().await?)
    }

    async fn post<Req, Resp>(&self, endpoint: &str, body: Option<&Req>) -> Result<Resp, Error>
    where
        Req: Serialize,
        Resp: DeserializeOwned,
    {
        let req = self.post_req_builder(endpoint, body);
        let response = self.send(req).await?;
        Ok(response.json().await?)
    }

    async fn post_text_resp<Req>(&self, endpoint: &str, body: Option<&Req>) -> Result<String, Error>
    where
        Req: Serialize,
    {
        let req = self.post_req_builder(endpoint, body);
        let response = self.send(req).await?;
        Ok(response.text().await?)
    }

    fn post_req_builder<Req>(&self, endpoint: &str, body: Option<&Req>) -> RequestBuilder
    where
        Req: Serialize,
    {
        let mut req = self
            .client
            .post(format!("https://api.getchange.io/api/v1/{0}", endpoint))
            .basic_auth(
                self.creds.public_key.to_string(),
                Some(self.creds.secret_key.to_string()),
            );
        if let Some(body) = body {
            req = req.json(body);
        }
        req
    }

    async fn send(&self, req: RequestBuilder) -> Result<Response, Error> {
        let response = req.send().await?;
        let status = response.status();
        if status.is_client_error() {
            let error_response = response.json::<ErrorResponse>().await?;
            Err(Error::ClientError {
                status,
                code: error_response.code,
                title: error_response.title,
            })
        } else {
            Ok(response)
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct ErrorResponse {
    pub status: u16,
    pub code: String,
    pub title: String,
}

#[derive(Serialize, Builder, Default, Debug)]
#[builder(setter(into))]
pub struct SearchNonprofitsRequest {
    #[builder(default)]
    search_term: Option<String>,

    #[builder(default)]
    page: Option<i32>,

    #[builder(default)]
    categories: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
pub struct SearchNonprofitsResponse {
    pub nonprofits: Vec<Nonprofit>,
    pub page: i32,
}

#[derive(Deserialize, Debug)]
pub struct Nonprofit {
    pub icon_url: String,
    pub id: String,
    pub name: String,
    pub ein: String,
    pub mission: String,
    pub display_impact: Vec<String>,
    pub category: String,

    #[serde(default)]
    pub email: Option<String>,

    #[serde(default)]
    pub pending_payment_amount: Option<i64>,
    // TODO: Add other fields
}

#[derive(Serialize, Builder, Default, Debug)]
#[builder(setter(into))]
pub struct CreateDonationRequest {
    amount: i64,
    nonprofit_id: String,
    funds_collected: bool,

    // Optional fields:
    #[builder(default)]
    count: Option<i64>,

    #[builder(default)]
    cover_fees: Option<bool>,

    #[builder(default)]
    external_id: Option<String>,

    #[builder(default)]
    account_id: Option<String>,

    #[builder(default)]
    order_value: Option<u64>,

    #[builder(default)]
    zip_code: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Donation {
    pub amount: i64,
    pub id: String,
    pub live_mode: bool,
    pub nonprofit_id: String,
    pub currency: String,

    // Optional fields:
    #[serde(default)]
    pub order_value: Option<u64>,

    #[serde(default)]
    pub zip_code: Option<String>,

    #[serde(default)]
    pub external_id: Option<String>,
}

#[derive(Serialize, Builder, Default, Debug)]
#[builder(setter(into))]
pub struct ListDonationsRequest {
    page: i32,

    // Optional fields:
    #[builder(default)]
    account_id: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ListDonationsResponse {
    pub donations: Vec<Donation>,
    pub page: i32,
}

#[derive(Serialize, Builder, Default, Debug)]
#[builder(setter(into))]
pub struct CreateAccountRequest {
    email: String,

    // Optional fields:
    #[builder(default)]
    name: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Account {
    pub id: String,
    pub active: bool,
    pub email: String,
    pub signed_agreement: bool,
    pub sign_page_url: String,
    pub saved_payment_method: bool,

    // Optional fields:
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Serialize, Builder, Default, Debug)]
#[builder(setter(into))]
pub struct AttachBankAccountRequest {
    link_token: String,
    plaid_public_token: String,
    bank_account_id: String,
}

#[derive(Serialize, Builder, Default, Debug)]
#[builder(setter(into))]
pub struct CreateLinkBankTokenRequest {
    id: String,
}
