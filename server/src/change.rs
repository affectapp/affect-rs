pub mod client;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("request error: {0:?}")]
    Http(#[from] reqwest::Error),

    #[error("json deserialization error: {0:?}")]
    Json(#[from] serde_json::Error),

    #[error("change client error: status={status}, code={code}, title={title}")]
    ClientError {
        status: hyper::StatusCode,
        code: String,
        title: String,
    },
}
