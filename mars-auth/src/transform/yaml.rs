use std::{error::Error, future::Future, pin::Pin};

use http::{header::CONTENT_LENGTH, Request, Response};
use serde_json::Value;
use tower::{Layer, Service};

use crate::response_from_status_message;

#[derive(Clone)]
pub struct YamlTransformJson<S> {
    inner: S,
    response_transform: bool,
    request_transform: bool,
}

pub struct YamlTransformJsonLayer {
    response_transform: bool,
    request_transform: bool,
}

impl<S> Layer<S> for YamlTransformJsonLayer {
    type Service = YamlTransformJson<S>;

    fn layer(&self, inner: S) -> Self::Service {
        YamlTransformJson {
            inner,
            response_transform: self.response_transform.clone(),
            request_transform: self.request_transform.clone(),
        }
    }
}

type ResBody = hyper::Body;
type ReqBody = hyper::Body;
impl<S> Service<Request<ReqBody>> for YamlTransformJson<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: 'static,
    S::Error: Send,
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
        let response_transfrom = self.response_transform.clone();
        let request_transform = self.request_transform.clone();
        let mut original = self.inner.clone();
        Box::pin(async move {
            let req = match get_transformed_request(req, request_transform).await {
                Ok(req) => req,
                Err(error) => {
                    return Ok(response_from_status_message(
                        500,
                        format!(
                            "unable to transform request because of this error {}",
                            error
                        ),
                    )
                    .unwrap())
                }
            };
            match original.call(req).await {
                Ok(resp) => {
                    let response = get_transformed_response(resp, response_transfrom).await;
                    match response {
                        Ok(resp) => Ok(resp),
                        Err(_) => {
                            let error = response.unwrap_err();
                            Ok(response_from_status_message(
                                500,
                                format!(
                                    "unable to transform response because of this error {}",
                                    error
                                ),
                            )
                            .unwrap())
                        }
                    }
                }
                Err(error) => return Err(error),
            }
        })
    }
}

async fn get_transformed_response(
    resp: Response<hyper::Body>,
    spec: bool,
) -> Result<Response<hyper::Body>, Box<dyn Error>> {
    if spec {
        let (mut parts, body) = resp.into_parts();
        let body = hyper::body::to_bytes(body).await?;
        let output_json: Value = serde_yaml::from_slice(&body)?;
        let out = serde_json::to_vec(&output_json)?;
        parts.headers.remove(CONTENT_LENGTH);
        Ok(Response::from_parts(parts, hyper::Body::from(out)))
    } else {
        Ok(resp)
    }
}

async fn get_transformed_request(
    request: Request<hyper::Body>,
    spec: bool,
) -> Result<Request<hyper::Body>, Box<dyn Error>> {
    if spec {
        let (mut parts, body) = request.into_parts();
        let body = hyper::body::to_bytes(body).await?;
        let output_json: Value = serde_yaml::from_slice(&body)?;
        let out = serde_json::to_vec(&output_json)?;
        parts.headers.remove(CONTENT_LENGTH);
        Ok(Request::from_parts(parts, hyper::Body::from(out)))
    } else {
        Ok(request)
    }
}

#[cfg(feature = "config")]
pub mod service_config {
    use mars_config::{MarsError, ServiceConfig};

    use super::YamlTransformJsonLayer;

    pub const TRANSFORM_YAML_JSON_REQUEST: &str = "request_yaml_to_json";
    pub const TRANSFORM_YAML_TO_JSON_RESPONSE: &str = "response_yaml_to_json";

    impl TryFrom<&ServiceConfig> for YamlTransformJsonLayer {
        type Error = MarsError;

        fn try_from(value: &ServiceConfig) -> Result<Self, Self::Error> {
            let request_transform = value
                .params
                .get_value(TRANSFORM_YAML_JSON_REQUEST)
                .and_then(|x| x.as_bool())
                .unwrap_or_default();
            let response_transform = value
                .params
                .get_value(TRANSFORM_YAML_TO_JSON_RESPONSE)
                .and_then(|x| x.as_bool())
                .unwrap_or_default();
            if !request_transform && !response_transform {
                return Err(MarsError::ServiceConfigError(
                    "both request_transform and response_transform failed not  avaiabile".into(),
                ));
            }
            Ok(YamlTransformJsonLayer {
                response_transform,
                request_transform,
            })
        }
    }
}
