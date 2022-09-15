use std::{convert::TryFrom, future::Future, pin::Pin};

use super::error::MarsError;

use super::config::ServiceConfig;

use http::{header::InvalidHeaderValue, HeaderValue, Request, Response};
use hyper::{client::HttpConnector, Client};
use hyper_tls::HttpsConnector;
use tower::{Layer, Service, ServiceBuilder};

#[derive(Clone)]
pub struct BasicAuth<S> {
    authentication: HeaderValue,
    pub inner: S,
}

pub struct BasicAuthLayer {
    authentication: HeaderValue,
}

#[allow(unused)]
impl BasicAuthLayer {
    pub fn new(authentication: HeaderValue) -> Self {
        BasicAuthLayer { authentication }
    }
}

impl<S> Layer<S> for BasicAuthLayer {
    type Service = BasicAuth<S>;

    fn layer(&self, inner: S) -> Self::Service {
        BasicAuth {
            authentication: self.authentication.clone(),
            inner: inner,
        }
    }
}

#[allow(unused)]
impl BasicAuthLayer {
    pub fn from_username_n_password(
        username: &str,
        password: &str,
    ) -> Result<BasicAuthLayer, InvalidHeaderValue> {
        let authentication = format!("{}:{}", username, password);
        let authentication = base64::encode(authentication);
        Ok(BasicAuthLayer::new(HeaderValue::from_str(&authentication)?))
    }
}

impl<ReqBody, ResBody, S> Service<Request<ReqBody>> for BasicAuth<S>
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
            .append("Authentication", self.authentication.clone());
        let fut = self.inner.call(req);
        Box::pin(async move { fut.await })
    }
}

impl TryFrom<&ServiceConfig> for BasicAuthLayer {
    type Error = MarsError;

    fn try_from(value: &ServiceConfig) -> Result<Self, Self::Error> {
        let username = value.get_handler_config("username")?;
        let password = value.get_handler_config("password")?;
        let basic_auth_layer = BasicAuthLayer::from_username_n_password(username, password)
            .map_err(|_| MarsError::ServiceConfigError)?;
        Ok(basic_auth_layer)
    }
}

impl TryFrom<&ServiceConfig> for BasicAuth<Client<HttpsConnector<HttpConnector>>> {
    type Error = MarsError;

    fn try_from(value: &ServiceConfig) -> Result<Self, Self::Error> {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);
        let auth_layer = BasicAuthLayer::try_from(value)?;
        let res = ServiceBuilder::new().layer(auth_layer).service(client);
        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use http::{Request, Response};
    use hyper::{client::HttpConnector, Client};
    use hyper_tls::HttpsConnector;
    use std::{future::Future, pin::Pin};
    use tower::{Service, ServiceBuilder};

    use super::{BasicAuth, BasicAuthLayer};

    pub trait ProxyService:
        Service<
        Request<hyper::Body>,
        Response = Response<hyper::Body>,
        Error = hyper::Error,
        Future = Pin<Box<dyn Future<Output = Result<Response<hyper::Body>, hyper::Error>> + Send>>,
    >
    {
    }

    impl ProxyService for BasicAuth<hyper::Client<HttpsConnector<HttpConnector>>> {}

    #[tokio::test]
    async fn basic_test() {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);
        let service = ServiceBuilder::new()
            .layer(BasicAuthLayer::from_username_n_password("prasanth", "prasanth").unwrap())
            .service(client);
        let mut service: Box<dyn ProxyService> = Box::new(service);
        let request = Request::builder()
            .uri("https://httpbin.org/get")
            .body(hyper::Body::empty())
            .unwrap();
        let response = service.call(request).await.unwrap();
        let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let s: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        let authentication = s
            .as_object()
            .unwrap()
            .get("headers")
            .unwrap()
            .as_object()
            .unwrap()
            .get("Authentication")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!("cHJhc2FudGg6cHJhc2FudGg=", authentication);
    }
}
