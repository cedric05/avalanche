use super::config::ServiceConfig;
use crate::error::MarsError;
use digest_auth::{AuthContext, HttpMethod};
use http::{HeaderValue, Request, Response};
use hyper::body;
use std::{future::Future, pin::Pin};
use tower::{Layer, Service};

// Credentials is not cloneable
#[derive(Clone)]
pub(crate) struct DigestAuth<S> {
    username: String,
    password: String,
    pub(crate) inner: S,
}

pub(crate) struct DigestAuthLayer {
    username: String,
    password: String,
}

#[allow(unused)]
impl DigestAuthLayer {}

impl<S> Layer<S> for DigestAuthLayer {
    type Service = DigestAuth<S>;

    fn layer(&self, inner: S) -> Self::Service {
        DigestAuth {
            username: self.username.clone(),
            password: self.password.clone(),
            inner,
        }
    }
}

type ResBody = hyper::Body;
type ReqBody = hyper::Body;
type HyperError = hyper::Error;
impl<S> Service<Request<ReqBody>> for DigestAuth<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>, Error = HyperError>
        + Send
        + 'static
        + Clone,
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
        let mut original = self.inner.clone();
        let (parts, body) = req.into_parts();
        let username = self.username.clone();
        let password = self.password.clone();
        Box::pin(async move {
            let uri = parts.uri.path_and_query().unwrap().to_string();
            let body = body::to_bytes(body).await?.to_vec();
            let signable = Request::builder()
                .method(parts.method.clone())
                .uri(parts.uri.clone())
                .version(parts.version)
                .body(hyper::body::Body::from(body.clone()))
                .unwrap();

            let initial_request = original.call(signable).await.unwrap();
            if initial_request.status() == 401 {
                let method = parts.method.to_string();
                let method = match method.as_str() {
                    "OPTIONS" => HttpMethod::OPTIONS,
                    "GET" => HttpMethod::GET,
                    "POST" => HttpMethod::POST,
                    "PUT" => HttpMethod::PUT,
                    "DELETE" => HttpMethod::DELETE,
                    "HEAD" => HttpMethod::HEAD,
                    "TRACE" => HttpMethod::TRACE,
                    "CONNECT" => HttpMethod::CONNECT,
                    "PATCH" => HttpMethod::PATCH,
                    _ => HttpMethod::GET,
                };
                let digest_context = AuthContext::new_with_method(
                    username,
                    password,
                    uri,
                    Some(body.clone()),
                    method,
                );
                let www_authenticate = initial_request
                    .headers()
                    .get("WWW-Authenticate")
                    .unwrap()
                    .to_str()
                    .unwrap();
                let mut prompt = digest_auth::parse(www_authenticate).unwrap();
                let answer = prompt.respond(&digest_context).unwrap().to_string();
                let mut rebuilt_request =
                    Request::from_parts(parts, hyper::body::Body::from(body.clone()));
                rebuilt_request
                    .headers_mut()
                    .insert("Authorization", HeaderValue::from_str(&answer).unwrap());
                original.call(rebuilt_request).await
            } else {
                Ok(Response::new(hyper::Body::empty()))
            }
        })
    }
}

impl TryFrom<&ServiceConfig> for DigestAuthLayer {
    type Error = MarsError;

    fn try_from(value: &ServiceConfig) -> Result<Self, Self::Error> {
        let username = value.get_handler_config("username")?;
        let password = value.get_handler_config("password")?;
        Ok(DigestAuthLayer {
            username: username.to_string(),
            password: password.to_string(),
        })
    }
}
