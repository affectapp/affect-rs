use async_trait::async_trait;
use futures::future::BoxFuture;
use hyper::{Body, Request, Response};
use std::task::{Context, Poll};
use tonic::{body::BoxBody, Status};
use tower::{Layer, Service};

#[async_trait]
pub trait AsyncInterceptor
where
    Self: Sized + Clone + Send + Sync,
{
    async fn intercept(&self, req: &mut Request<Body>) -> Result<(), Status>;
}

#[derive(Clone, Debug)]
pub struct AsyncInterceptorLayer<I>
where
    I: AsyncInterceptor,
{
    interceptor: I,
}

impl<I> AsyncInterceptorLayer<I>
where
    I: AsyncInterceptor,
{
    pub fn new(interceptor: I) -> Self {
        Self { interceptor }
    }
}

impl<S, I> Layer<S> for AsyncInterceptorLayer<I>
where
    I: AsyncInterceptor,
{
    type Service = AsyncInterceptorService<S, I>;

    fn layer(&self, service: S) -> Self::Service {
        Self::Service {
            inner: service,
            interceptor: self.interceptor.clone(),
        }
    }
}

impl<I> From<I> for AsyncInterceptorLayer<I>
where
    I: AsyncInterceptor,
{
    fn from(interceptor: I) -> Self {
        Self::new(interceptor)
    }
}

#[derive(Clone, Debug)]
pub struct AsyncInterceptorService<S, I>
where
    I: AsyncInterceptor,
{
    inner: S,
    interceptor: I,
}

impl<S, I> Service<hyper::Request<Body>> for AsyncInterceptorService<S, I>
where
    S: Service<hyper::Request<Body>, Response = Response<BoxBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    I: AsyncInterceptor + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: hyper::Request<Body>) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        let interceptor = self.interceptor.clone();

        Box::pin(async move {
            let middleware_result = interceptor.intercept(&mut req).await;

            match middleware_result {
                Ok(_) => {
                    let response = inner.call(req).await?;
                    Ok(response)
                }
                Err(status) => Ok(status.to_http()),
            }
        })
    }
}
