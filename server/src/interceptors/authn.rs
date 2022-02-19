use crate::async_interceptor::AsyncInterceptor;
use crate::firebase::FirebaseAuth;
use affect_api::affect::auth_metadata::PeerToken;
use affect_api::affect::AuthMetadata;
use affect_storage::user::{UserRow, UserStore};
use async_trait::async_trait;
use hyper::Request;
use hyper::{header::HeaderValue, Body};
use prost::Message;
use std::io::Cursor;
use std::sync::Arc;
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

    async fn authenticate(&self, req: &hyper::Request<Body>) -> Result<Peer, Status> {
        let auth_metadata_from_bytes = req
            .headers()
            .get("auth-bin")
            .map(HeaderValue::as_ref)
            .map(|u8| AuthMetadata::decode(&mut Cursor::new(u8)))
            .transpose()
            .map_err(|_| Status::unauthenticated("'auth-bin' header could not be decoded"))?;

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
                        .get_user_by_firebase_uid(decoded_id_token.uid)
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

        println!("Request from {:?}", peer);

        Ok(peer)
    }
}

#[async_trait]
impl AsyncInterceptor for AuthnInterceptor {
    async fn intercept(&self, req: &mut Request<Body>) -> Result<(), Status> {
        let peer = self.authenticate(req).await?;
        req.extensions_mut().insert(peer);
        Ok(())
    }
}
