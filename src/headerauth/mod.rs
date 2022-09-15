use std::{future::Future, pin::Pin, str::FromStr};

use crate::error::MarsError;

use super::config::ServiceConfig;
use http::{
    header::{HeaderName, InvalidHeaderValue},
    HeaderMap, HeaderValue, Request, Response,
};
use hyper::{client::HttpConnector, Client};
use hyper_tls::HttpsConnector;
use tower::{Layer, Service, ServiceBuilder};

#[derive(Clone)]
pub struct HeaderAuth<S> {
    header_map: HeaderMap<HeaderValue>,
    pub inner: S,
}

pub struct HeaderAuthLayer {
    header_map: HeaderMap<HeaderValue>,
}

#[allow(unused)]
impl HeaderAuthLayer {
    pub fn new(header_map: HeaderMap<HeaderValue>) -> Self {
        HeaderAuthLayer { header_map }
    }
}

impl<S> Layer<S> for HeaderAuthLayer {
    type Service = HeaderAuth<S>;

    fn layer(&self, inner: S) -> Self::Service {
        HeaderAuth {
            header_map: self.header_map.clone(),
            inner: inner,
        }
    }
}

#[allow(unused)]
impl HeaderAuthLayer {
    pub fn from_header(
        header_map: HeaderMap<HeaderValue>,
    ) -> Result<HeaderAuthLayer, InvalidHeaderValue> {
        Ok(HeaderAuthLayer::new(header_map))
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
        req.headers_mut().extend(self.header_map.clone());
        let fut = self.inner.call(req);
        Box::pin(async move { fut.await })
    }
}

impl TryFrom<&ServiceConfig> for HeaderAuthLayer {
    type Error = MarsError;

    fn try_from(value: &ServiceConfig) -> Result<Self, Self::Error> {
        let mut header_map = HeaderMap::new();
        for header_pair in value
            .handler
            .params
            .as_array()
            .ok_or(MarsError::ServiceConfigError)?
        {
            let pair = header_pair
                .as_object()
                .ok_or(MarsError::ServiceConfigError)?;
            let value = pair
                .get("key")
                .ok_or(MarsError::ServiceConfigError)?
                .as_str()
                .ok_or(MarsError::ServiceConfigError)?;
            let header_key =
                HeaderName::from_str(value).map_err(|_| MarsError::ServiceConfigError)?;
            let value = pair
                .get("value")
                .ok_or(MarsError::ServiceConfigError)?
                .as_str()
                .ok_or(MarsError::ServiceConfigError)?;
            let header_value =
                HeaderValue::from_str(value).map_err(|_| MarsError::ServiceConfigError)?;
            header_map.insert(header_key, header_value);
        }
        let header_auth_layer =
            HeaderAuthLayer::from_header(header_map).map_err(|_| MarsError::ServiceConfigError)?;
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
