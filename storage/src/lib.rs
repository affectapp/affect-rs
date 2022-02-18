use affect_status::Status;

pub mod user;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("sql failed")]
    Sql(#[from] sqlx::Error),
}

impl From<Error> for Status {
    fn from(error: Error) -> Self {
        return Status::internal(format!("storage error: {0}", error));
    }
}
