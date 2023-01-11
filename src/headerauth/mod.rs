use std::{future::Future, pin::Pin, str::FromStr};

use crate::error::MarsError;

use super::config::ServiceConfig;
use http::{header::HeaderName, HeaderMap, HeaderValue, Request, Response};
use serde::{Deserialize, Serialize};
use tower::{Layer, Service};

#[derive(Clone)]
pub(crate) struct HeaderAuth<S> {
    header_map: HeaderMap<HeaderValue>,
    pub(crate) inner: S,
}

pub(crate) struct HeaderAuthLayer {
    header_map: HeaderMap<HeaderValue>,
}

#[allow(unused)]
impl HeaderAuthLayer {
    pub(crate) fn new(header_map: HeaderMap<HeaderValue>) -> Self {
        HeaderAuthLayer { header_map }
    }
}

impl<S> Layer<S> for HeaderAuthLayer {
    type Service = HeaderAuth<S>;

    fn layer(&self, inner: S) -> Self::Service {
        HeaderAuth {
            header_map: self.header_map.clone(),
            inner,
        }
    }
}

#[allow(unused)]
impl HeaderAuthLayer {
    pub(crate) fn from_header(header_map: HeaderMap<HeaderValue>) -> HeaderAuthLayer {
        HeaderAuthLayer::new(header_map)
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

#[derive(Serialize, Deserialize)]
struct HeadersAuthConfig(Vec<HeaderAuthConfig>);

#[derive(Serialize, Deserialize)]

struct HeaderAuthConfig {
    key: String,
    value: String,
}

impl TryFrom<HeaderAuthConfig> for (HeaderName, HeaderValue) {
    type Error = MarsError;

    fn try_from(header_config: HeaderAuthConfig) -> Result<Self, Self::Error> {
        Ok((
            HeaderName::from_str(&header_config.key).map_err(|x| {
                MarsError::ServiceConfigError(format!(
                    "unable to crate header name from {} because of {x}",
                    header_config.key
                ))
            })?,
            HeaderValue::from_str(&header_config.value).map_err(|x| {
                MarsError::ServiceConfigError(format!(
                    "unable to crate header value from {} because of {x}",
                    header_config.value
                ))
            })?,
        ))
    }
}

impl TryFrom<HeadersAuthConfig> for HeaderMap {
    type Error = MarsError;

    fn try_from(value: HeadersAuthConfig) -> Result<Self, Self::Error> {
        let mut header_map = HeaderMap::default();
        for x in value.0 {
            let (name, value) = TryInto::<(HeaderName, HeaderValue)>::try_into(x)?;
            header_map.insert(name, value);
        }
        Ok(header_map)
    }
}

impl TryFrom<&ServiceConfig> for HeaderAuthLayer {
    type Error = MarsError;

    fn try_from(value: &ServiceConfig) -> Result<Self, Self::Error> {
        let headers_auth_config: HeadersAuthConfig =
            serde_json::from_value(value.auth.params.clone()).map_err(|err| {
                MarsError::ServiceConfigError(format!(
                    "unable to parse headers auth config because of  {err}"
                ))
            })?;
        let header_map = TryInto::<HeaderMap>::try_into(headers_auth_config)?;
        let header_auth_layer = HeaderAuthLayer::from_header(header_map);
        Ok(header_auth_layer)
    }
}
