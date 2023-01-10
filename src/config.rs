use std::{error::Error, future::Future, pin::Pin, str::FromStr};

use http::{header::HeaderName, HeaderValue, Request, Response, Uri};
use serde::{Deserialize, Serialize};

use tower::{Layer, Service};
use url::Url;

use crate::{error::MarsError, project::AVALANCHE_TOKEN};
pub(crate) use mars_config::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ServiceConfig {
    pub(crate) url: String,
    pub(crate) method: Method,
    pub(crate) query_params: Vec<UrlParam>,
    pub(crate) headers: Vec<Header>,
    pub(crate) handler: ProxyParams,
}

lazy_static::lazy_static! {
    static ref HOP_HEADERS: Vec<HeaderName> = vec![
        HeaderName::from_str("Connection").unwrap(),
        HeaderName::from_str("Keep-Alive").unwrap(),
        HeaderName::from_str("Proxy-Authenticate").unwrap(),
        HeaderName::from_str("Proxy-Authorization").unwrap(),
        HeaderName::from_str("Te").unwrap(),
        HeaderName::from_str("Trailers").unwrap(),
        HeaderName::from_str("Transfer-Encoding").unwrap(),
        HeaderName::from_str("Upgrade").unwrap(),
        HeaderName::from_str("Host").unwrap(),
    ];
}
impl ServiceConfig {
    pub(crate) fn get_handler_config(&self, key: &str) -> Result<&str, MarsError> {
        self.handler
            .params
            .get(key)
            .ok_or_else(|| {
                MarsError::ServiceConfigError(format!("service config `{}` not found", key))
            })?
            .as_str()
            .ok_or_else(|| {
                MarsError::ServiceConfigError(format!(
                    "service config `{}` found but not string",
                    key
                ))
            })
    }

    #[allow(unused)]
    pub(crate) fn get_timeout(&self) -> Option<f64> {
        self.handler.params.get("timeout").and_then(|x| x.as_f64())
    }

    pub(crate) fn get_updated_request<ReqBody>(
        &self,
        rest: &str,
        req: &mut Request<ReqBody>,
    ) -> Result<(), Box<dyn Error>> {
        let uri = Url::from_str(&self.url.clone())?;
        let uri = uri.join(rest)?;
        let params: Vec<(String, String)> = self
            .query_params
            .iter()
            .filter(|x| x.action == Action::Add)
            .map(|x| (x.key.clone(), x.value.clone()))
            .collect();
        let uri: Url = Url::parse_with_params(uri.as_ref(), &params)?;
        *req.uri_mut() = Uri::from_str(uri.as_ref())?;
        let headers_mut = req.headers_mut();
        headers_mut.remove(AVALANCHE_TOKEN);
        for header in &self.headers {
            if header.action == Action::Add {
                let key = HeaderName::from_str(&header.key)?;
                headers_mut.append(key, HeaderValue::from_str(&header.value)?);
            }
        }
        for header in HOP_HEADERS.iter() {
            headers_mut.remove(header);
        }
        Ok(())
    }
}

pub(crate) struct ProxyUrlPath(pub String);

#[derive(Clone)]
pub(crate) struct CommonUpdateQueryNHeaders<S> {
    service_config: ServiceConfig,
    inner: S,
}

#[derive(Clone)]
pub(crate) struct CommonUpdateQueryNHeaderLayer {
    service_config: ServiceConfig,
}

impl CommonUpdateQueryNHeaderLayer {
    pub(crate) fn new(service_config: ServiceConfig) -> Self {
        Self {
            service_config: service_config,
        }
    }
}

impl<S> Layer<S> for CommonUpdateQueryNHeaderLayer {
    type Service = CommonUpdateQueryNHeaders<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CommonUpdateQueryNHeaders {
            service_config: self.service_config.clone(),
            inner,
        }
    }
}

impl<ReqBody, ResBody, S> Service<Request<ReqBody>> for CommonUpdateQueryNHeaders<S>
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

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let url: Option<String> = req.extensions().get::<ProxyUrlPath>().map(|x| x.0.clone());
        self.service_config
            .get_updated_request(&url.unwrap(), &mut req)
            .unwrap();
        let fut = self.inner.call(req);
        Box::pin(async move { fut.await })
    }
}
