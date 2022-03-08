pub mod page_token;
pub mod sqlx;
pub mod stores;

// Necessary since module has same name as the lib.
extern crate sqlx as sqlx_lib;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    // Error occurred when executing some SQL operation.
    #[error("sql failed: {0:?}")]
    Sql(#[from] sqlx_lib::Error),

    // Error occurred when running migrations.
    #[error("migration failed: {0:?}")]
    Migrate(#[from] sqlx_lib::migrate::MigrateError),

    // Page token serialization/deserialization failed.
    #[error(transparent)]
    PageToken(anyhow::Error),

    // Some other/unexpected error occurred.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<Error> for affect_status::Status {
    fn from(error: Error) -> Self {
        return affect_status::Status::internal(format!("storage error: {:?}", error));
    }
}
