use std::{future::Future, pin::Pin};

use crate::error::MarsError;

use super::config::ServiceConfig;
use hawk::{Credentials, DigestAlgorithm, Key, RequestBuilder};
use http::{HeaderValue, Request, Response};
use hyper::{client::HttpConnector, Client};
use hyper_tls::HttpsConnector;
use tower::{Layer, Service, ServiceBuilder};

// Credentials is not cloneable
#[derive(Clone)]
pub struct HawkAuth<S> {
    id: String,
    key: String,
    algorithm: DigestAlgorithm,
    pub inner: S,
}

pub struct HawkAuthLayer {
    id: String,
    key: String,
    algorithm: DigestAlgorithm,
}

#[allow(unused)]
impl HawkAuthLayer {}

impl<S> Layer<S> for HawkAuthLayer {
    type Service = HawkAuth<S>;

    fn layer(&self, inner: S) -> Self::Service {
        HawkAuth {
            id: self.id.clone(),
            key: self.key.clone(),
            algorithm: self.algorithm.clone(),
            inner: inner,
        }
    }
}

impl<ReqBody, ResBody, S> Service<Request<ReqBody>> for HawkAuth<S>
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
        let credentials = Credentials {
            id: self.id.clone(),
            key: Key::new(self.key.clone(), self.algorithm).unwrap(),
        };
        let method = req.method().as_str();
        let host = req.uri().host().unwrap();
        let port = match (req.uri().port(), req.uri().scheme()) {
            (Some(port), _) => port.as_u16(),
            (None, Some(scheme)) => {
                let scheme = scheme.as_str();
                if scheme == "https" {
                    443
                } else {
                    80
                }
            }
            (None, None) => 80,
        };
        let path = req.uri().path();
        let client_req = RequestBuilder::new(method, host, port, path).request();
        let header = client_req.make_header(&credentials).unwrap();
        let val = HeaderValue::from_str(header.to_string().as_str()).unwrap();
        req.headers_mut().insert("Authentication", val);
        let fut = self.inner.call(req);
        Box::pin(async move { fut.await })
    }
}

impl TryFrom<&ServiceConfig> for HawkAuthLayer {
    type Error = MarsError;

    fn try_from(value: &ServiceConfig) -> Result<Self, Self::Error> {
        let key = value.get_handler_config("key")?;
        let id = value.get_handler_config("id")?;
        let algorithm = value.get_handler_config("algorithm")?;
        let algorithm = match algorithm {
            "sha256" => DigestAlgorithm::Sha256,
            "sha384" => DigestAlgorithm::Sha384,
            "sha512" => DigestAlgorithm::Sha512,
            _ => return Err(MarsError::ServiceConfigError),
        };
        Ok(HawkAuthLayer {
            id: id.to_string(),
            key: key.to_string(),
            algorithm: algorithm,
        })
    }
}

impl TryFrom<&ServiceConfig> for HawkAuth<Client<HttpsConnector<HttpConnector>>> {
    type Error = MarsError;

    fn try_from(value: &ServiceConfig) -> Result<Self, Self::Error> {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);
        let auth_layer = HawkAuthLayer::try_from(value)?;
        let res = ServiceBuilder::new().layer(auth_layer).service(client);
        Ok(res)
    }
}
