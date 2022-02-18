use chrono::{DateTime, TimeZone, Utc};
use jwks_client::keyset::KeyStore;
use serde::Deserialize;

pub struct FirebaseAuth {
    key_store: KeyStore,
    project_id: String,
}

pub struct DecodedIdToken {
    pub uid: String,
    pub email: String,
    pub email_verified: bool,
    pub auth_time: DateTime<Utc>,
    pub expire_time: DateTime<Utc>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("could not decode: {0}")]
    Decode(#[from] jwks_client::error::Error),

    #[error("invalid firebase project: {0}")]
    InvalidProject(String),
}

impl FirebaseAuth {
    pub fn new(key_store: KeyStore, project_id: String) -> Self {
        Self {
            key_store,
            project_id,
        }
    }

    pub async fn load(
        gwk_url: String,
        project_id: String,
    ) -> Result<Self, jwks_client::error::Error> {
        let key_store = KeyStore::new_from(gwk_url).await?;
        Ok(Self::new(key_store, project_id))
    }

    pub fn verify_id_token(&self, id_token: String) -> Result<DecodedIdToken, Error> {
        let jwt = self.key_store.decode(&id_token)?;
        let claims = jwt.payload().into::<Claims>()?;
        if claims.aud != self.project_id {
            return Err(Error::InvalidProject(claims.aud));
        }

        Ok(DecodedIdToken {
            uid: claims.user_id,
            email: claims.email,
            email_verified: claims.email_verified,
            auth_time: Utc.timestamp(claims.auth_time, 0),
            expire_time: Utc.timestamp(claims.exp, 0),
        })
    }
}

#[derive(Deserialize, Debug)]
struct Claims {
    // The audience the token was issued for
    pub aud: String,
    // User id
    pub user_id: String,
    // User email
    pub email: String,
    // Email verified
    pub email_verified: bool,
    // Auth time (epoch seconds)
    pub auth_time: i64,
    // The expiry date -- as epoch seconds
    pub exp: i64,
}
