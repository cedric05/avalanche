use std::{future::Future, pin::Pin};

use http::{HeaderValue, Request, Response};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct BasicAuth<S> {
    username: String,
    password: String,
    pub inner: S,
}

pub struct BasicAuthLayer {
    username: String,
    password: String,
}

impl BasicAuthLayer {
    pub fn new(username: String, password: String) -> Self {
        BasicAuthLayer { username, password }
    }
}

impl<S> Layer<S> for BasicAuthLayer {
    type Service = BasicAuth<S>;

    fn layer(&self, inner: S) -> Self::Service {
        BasicAuth {
            username: self.username.to_string(),
            password: self.password.to_string(),
            inner: inner,
        }
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
        req.headers_mut().append(
            "Authentication",
            HeaderValue::from_str(&base64::encode(format!(
                "{}:{}",
                self.username, self.password
            )))
            .unwrap(),
        );
        let fut = self.inner.call(req);
        Box::pin(async move { fut.await })
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
            .layer(BasicAuthLayer::new(
                "prasanth".to_string(),
                "prasanth".to_string(),
            ))
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
