use std::{future::Future, pin::Pin, str::FromStr};

use http::{Request, Response};
use hyper::Body;
use mars_config::{ServiceConfig, AVALANCHE_TOKEN};
use tower::{Layer, Service};

use http::{header::HeaderName, HeaderValue, Uri};
use mars_config::Action;
use std::error::Error;

use url::Url;

pub fn response_from_status_message(
    status: u16,
    message: String,
) -> Result<Response<Body>, Box<dyn Error>> {
    Ok(Response::builder()
        .status(status)
        .body(Body::from(message))?)
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

pub struct ProxyUrlPath(pub String);

#[derive(Clone)]
pub struct CommonUpdateQueryNHeaders<S> {
    service_config: ServiceConfig,
    inner: S,
}

impl<S> CommonUpdateQueryNHeaders<S> {
    pub(crate) fn get_updated_request<ReqBody>(
        &self,
        rest: &str,
        req: &mut Request<ReqBody>,
    ) -> Result<(), Box<dyn Error>> {
        let uri = Url::from_str(&self.service_config.url.clone())?;
        let uri = uri.join(rest)?;
        let params: Vec<(String, String)> = self
            .service_config
            .query_params
            .iter()
            .filter(|x| x.action == Action::Add)
            .map(|x| (x.key.clone(), x.value.clone()))
            .collect();
        let uri: Url = Url::parse_with_params(uri.as_ref(), &params)?;
        *req.uri_mut() = Uri::from_str(uri.as_ref())?;
        let headers_mut = req.headers_mut();
        headers_mut.remove(AVALANCHE_TOKEN);
        for header in &self.service_config.headers {
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

#[derive(Clone)]
pub struct CommonUpdateQueryNHeaderLayer {
    service_config: ServiceConfig,
}

impl CommonUpdateQueryNHeaderLayer {
    pub fn new(service_config: ServiceConfig) -> Self {
        Self { service_config }
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

type ResBody = hyper::Body;
impl<ReqBody, S> Service<Request<ReqBody>> for CommonUpdateQueryNHeaders<S>
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
        match self.get_updated_request(&url.expect("impossible to fail"), &mut req) {
            Ok(_) => {
                let fut = self.inner.call(req);
                Box::pin(async move { fut.await })
            }
            Err(error) => {
                let message = format!("unable to transform request err: `{:?}`", error);
                Box::pin(async move {
                    Ok(response_from_status_message(500, message).expect("impossible to fail"))
                })
            }
        }
    }
}
