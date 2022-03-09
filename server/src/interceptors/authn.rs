use crate::{firebase::FirebaseAuth, tonic::async_interceptor::AsyncInterceptor};
use affect_api::affect::{auth_metadata::PeerToken, AuthMetadata};
use affect_storage::stores::user::{UserRow, UserStore};
use async_trait::async_trait;
use hyper::{Body, Request};
use log::debug;
use prost::Message;
use std::{io::Cursor, sync::Arc};
use tonic::Status;

#[derive(Clone, Debug)]
pub enum Peer {
    User(UserRow),
    Privileged(UserRow),
    Impersonated {
        user: UserRow,
        privileged_user: UserRow,
    },
    Anonymous,
}

impl Peer {}

#[derive(Clone)]
pub struct AuthnInterceptor {
    firebase_auth: Arc<FirebaseAuth>,
    user_store: Arc<dyn UserStore>,
}

impl AuthnInterceptor {
    pub fn new(firebase_auth: Arc<FirebaseAuth>, user_store: Arc<dyn UserStore>) -> Self {
        Self {
            firebase_auth,
            user_store,
        }
    }

    pub async fn authenticate_bytes(&self, u8: Option<&[u8]>) -> Result<Peer, Status> {
        let auth_metadata_from_bytes = u8
            .map(|u8| AuthMetadata::decode(&mut Cursor::new(u8)))
            .transpose()
            .map_err(|e| {
                Status::unauthenticated(format!("'auth-bin' header could not be decoded: {:?}", e))
            })?;

        let peer = match auth_metadata_from_bytes {
            Some(auth_metadata) => match auth_metadata.peer_token {
                Some(PeerToken::EndUser(end_user)) => {
                    let decoded_id_token = self
                        .firebase_auth
                        .verify_id_token(end_user.firebase_id_token)
                        .map_err(|_| {
                            Status::unauthenticated("failed to decoded end user firebase id token")
                        })?;
                    let user_row = self
                        .user_store
                        .find_user_by_firebase_uid(decoded_id_token.uid)
                        .await?
                        .ok_or(Status::unauthenticated("end user not found"))?;
                    Peer::User(user_row)
                }
                Some(PeerToken::Privileged(_)) => Peer::Anonymous,
                Some(PeerToken::ImpersonatedUser(_)) => Peer::Anonymous,
                Some(PeerToken::Anonymous(_)) => Peer::Anonymous,
                None => Peer::Anonymous,
            },
            _ => Peer::Anonymous,
        };

        debug!("Request from {:?}", peer);

        Ok(peer)
    }

    pub async fn authenticate_hyper_request(
        &self,
        req: &hyper::Request<Body>,
    ) -> Result<Peer, Status> {
        let base64_bytes = match req.headers().get("auth-bin").map(|value| value.as_ref()) {
            Some(bytes) => bytes,
            None => return self.authenticate_bytes(None).await,
        };

        let base64_string =
            std::str::from_utf8(base64_bytes).map_err(|_| Status::unauthenticated("not utf8"))?;
        let auth_metadata_bytes =
            base64::decode(base64_string).map_err(|_| Status::unauthenticated("not base64"))?;

        self.authenticate_bytes(Some(&auth_metadata_bytes)).await
    }

    pub async fn authenticate_tonic_request<T>(
        &self,
        req: &tonic::Request<T>,
    ) -> Result<Peer, Status> {
        let base64_bytes = match req
            .metadata()
            .get_bin("auth-bin")
            .map(|value| value.as_ref())
        {
            Some(bytes) => bytes,
            None => return self.authenticate_bytes(None).await,
        };

        let base64_string =
            std::str::from_utf8(base64_bytes).map_err(|_| Status::unauthenticated("not utf8"))?;
        let auth_metadata_bytes =
            base64::decode(base64_string).map_err(|_| Status::unauthenticated("not base64"))?;

        self.authenticate_bytes(Some(&auth_metadata_bytes)).await
    }
}

#[async_trait]
impl AsyncInterceptor for AuthnInterceptor {
    async fn intercept(&self, req: &mut Request<Body>) -> Result<(), Status> {
        let peer = self.authenticate_hyper_request(req).await?;
        req.extensions_mut().insert(peer);
        Ok(())
    }
}
