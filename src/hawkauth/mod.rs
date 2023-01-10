use std::{future::Future, pin::Pin};

use crate::error::MarsError;

use super::config::ServiceConfig;
use hawk::{Credentials, DigestAlgorithm, Key, RequestBuilder};
use http::{HeaderValue, Request, Response};
use tower::{Layer, Service};
use url::Url;

// Credentials is not cloneable
#[derive(Clone)]
pub(crate) struct HawkAuth<S> {
    id: String,
    key: String,
    algorithm: DigestAlgorithm,
    pub(crate) inner: S,
}

pub(crate) struct HawkAuthLayer {
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
            algorithm: self.algorithm,
            inner,
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
        let url = Url::parse(&req.uri().to_string()).unwrap();
        let client_req = RequestBuilder::from_url(req.method().as_str(), &url)
            .unwrap()
            .request();
        let header = client_req.make_header(&credentials).unwrap();
        req.headers_mut().insert(
            "Authorization",
            HeaderValue::from_str(&format!("Hawk {}", header)).unwrap(),
        );
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
            _ => {
                return Err(MarsError::ServiceConfigError(
                    "unsupported algorithm for hawk auth".to_string(),
                ))
            }
        };
        Ok(HawkAuthLayer {
            id: id.to_string(),
            key: key.to_string(),
            algorithm,
        })
    }
}
