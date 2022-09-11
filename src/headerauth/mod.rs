use std::{error::Error, future::Future, pin::Pin, str::FromStr};

use crate::error::MarsError;

use super::config::ServiceConfig;
use super::project::ProxyService;
use async_trait::async_trait;
use http::{
    header::{HeaderName, InvalidHeaderValue},
    HeaderValue, Request, Response,
};
use hyper::{client::HttpConnector, Body, Client};
use hyper_tls::HttpsConnector;
use tower::{Layer, Service, ServiceBuilder};

#[derive(Clone)]
pub struct HeaderAuth<S> {
    key: HeaderName,
    value: HeaderValue,
    pub inner: S,
}

pub struct HeaderAuthLayer {
    key: HeaderName,
    value: HeaderValue,
}

#[allow(unused)]
impl HeaderAuthLayer {
    pub fn new(key: HeaderName, value: HeaderValue) -> Self {
        HeaderAuthLayer { key, value }
    }
}

impl<S> Layer<S> for HeaderAuthLayer {
    type Service = HeaderAuth<S>;

    fn layer(&self, inner: S) -> Self::Service {
        HeaderAuth {
            key: self.key.clone(),
            value: self.value.clone(),
            inner: inner,
        }
    }
}

#[allow(unused)]
impl HeaderAuthLayer {
    pub fn from_header(key: &str, value: &str) -> Result<HeaderAuthLayer, InvalidHeaderValue> {
        let key = HeaderName::from_str(key).unwrap();
        let value = HeaderValue::from_str(value).unwrap();
        Ok(HeaderAuthLayer::new(key, value))
    }
}

impl<ReqBody, ResBody, S> Service<Request<ReqBody>> for HeaderAuth<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    S::Future: 'static,
    <S as Service<Request<ReqBody>>>::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let mut req = req;
        req.headers_mut()
            .append(self.key.clone(), self.value.clone());
        let fut = self.inner.call(req);
        Box::pin(async move { fut.await })
    }
}

#[async_trait]
impl ProxyService for HeaderAuth<hyper::Client<HttpsConnector<HttpConnector>>> {
    async fn handle_service(
        &mut self,
        url: &str,
        service_config: &ServiceConfig,
        request: hyper::Request<Body>,
    ) -> Result<Response<Body>, Box<dyn Error>> {
        let mut request = request;
        service_config.get_updated_request(url, &mut request)?;
        let response = self.call(request).await?;
        Ok(response)
    }
}

impl TryFrom<&ServiceConfig> for HeaderAuthLayer {
    type Error = MarsError;

    fn try_from(value: &ServiceConfig) -> Result<Self, Self::Error> {
        let key = value
            .handler
            .params
            .get("key")
            .ok_or(MarsError::ServiceConfigError)?
            .as_str()
            .ok_or(MarsError::ServiceConfigError)?;
        let value = value
            .handler
            .params
            .get("value")
            .ok_or(MarsError::ServiceConfigError)?
            .as_str()
            .ok_or(MarsError::ServiceConfigError)?;
        let header_auth_layer =
            HeaderAuthLayer::from_header(key, value).map_err(|_| MarsError::ServiceConfigError)?;
        Ok(header_auth_layer)
    }
}

impl TryFrom<&ServiceConfig> for HeaderAuth<Client<HttpsConnector<HttpConnector>>> {
    type Error = MarsError;

    fn try_from(value: &ServiceConfig) -> Result<Self, Self::Error> {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);
        let auth_layer = HeaderAuthLayer::try_from(value)?;
        let res = ServiceBuilder::new().layer(auth_layer).service(client);
        Ok(res)
    }
}
