use std::{future::Future, pin::Pin};

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
#[cfg(feature = "config")]
pub mod service_config {
    use super::HawkAuthLayer;
    use hawk::DigestAlgorithm;
    use mars_config::{MarsError, ServiceConfig};
    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize)]
    struct HawkAuthParams {
        key: String,
        id: String,
        algorithm: Algorithm,
    }

    #[derive(Serialize, Deserialize)]

    enum Algorithm {
        #[serde(rename = "sha256")]
        Sha256,
        #[serde(rename = "sha384")]
        Sha384,
        #[serde(rename = "sha512")]
        Sha512,
    }

    impl TryFrom<&ServiceConfig> for HawkAuthLayer {
        type Error = MarsError;

        fn try_from(value: &ServiceConfig) -> Result<Self, Self::Error> {
            let hawk_auth_params: HawkAuthParams = serde_json::from_value(value.auth.get_params())
                .map_err(|err| {
                    MarsError::ServiceConfigError(format!(
                        "unable to parse auth params for hawk auth configuration error:{}",
                        err
                    ))
                })?;
            let algorithm = match hawk_auth_params.algorithm {
                Algorithm::Sha256 => DigestAlgorithm::Sha256,
                Algorithm::Sha384 => DigestAlgorithm::Sha384,
                Algorithm::Sha512 => DigestAlgorithm::Sha512,
            };
            Ok(HawkAuthLayer {
                id: hawk_auth_params.id,
                key: hawk_auth_params.key,
                algorithm,
            })
        }
    }
}
