use std::{error::Error, future::Future, pin::Pin};

use fluvio_jolt::TransformSpec;
use http::{
    header::{CONTENT_LENGTH, CONTENT_TYPE},
    Request, Response,
};
use serde_json::Value;
use tower::{Layer, Service};

use crate::response_from_status_message;

#[derive(Clone)]
pub struct JoltTransform<S> {
    inner: S,
    response_transform: Option<TransformSpec>,
    request_transform: Option<TransformSpec>,
}

pub struct JoltTransformLayer {
    response_transform: Option<TransformSpec>,
    request_transform: Option<TransformSpec>,
}

impl<S> Layer<S> for JoltTransformLayer {
    type Service = JoltTransform<S>;

    fn layer(&self, inner: S) -> Self::Service {
        JoltTransform {
            inner,
            response_transform: self.response_transform.clone(),
            request_transform: self.request_transform.clone(),
        }
    }
}

type ResBody = hyper::Body;
type ReqBody = hyper::Body;
impl<S> Service<Request<ReqBody>> for JoltTransform<S>
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
        // std::mem::replace(&mut self.inner, original);
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
    spec: Option<TransformSpec>,
) -> Result<Response<hyper::Body>, Box<dyn Error>> {
    if let Some(spec) = spec {
        let (mut parts, body) = resp.into_parts();
        let body = hyper::body::to_bytes(body).await?;
        let value: Value = serde_json::from_slice(&body)?;
        let out = fluvio_jolt::transform(value, &spec);
        let out = serde_json::to_vec(&out)?;
        parts.headers.remove(CONTENT_LENGTH);
        parts
            .headers
            .insert(CONTENT_TYPE, "application/json".try_into()?);
        Ok(Response::from_parts(parts, hyper::Body::from(out)))
    } else {
        Ok(resp)
    }
}

async fn get_transformed_request(
    request: Request<hyper::Body>,
    spec: Option<TransformSpec>,
) -> Result<Request<hyper::Body>, Box<dyn Error>> {
    if let Some(spec) = spec {
        let (mut parts, body) = request.into_parts();
        let body = hyper::body::to_bytes(body).await?;
        let value: Value = serde_json::from_slice(&body)?;
        let out = fluvio_jolt::transform(value, &spec);
        let out = serde_json::to_vec(&out)?;
        parts.headers.remove(CONTENT_LENGTH);
        parts
            .headers
            .insert(CONTENT_TYPE, "application/json".try_into()?);
        Ok(Request::from_parts(parts, hyper::Body::from(out)))
    } else {
        Ok(request)
    }
}

#[cfg(feature = "config")]
pub mod service_config {
    use mars_config::{MarsError, ServiceConfig};
    pub use mars_config::{
        TRANSFORM_JSON_TO_JSON_JOLT_REQUEST, TRANSFORM_JSON_TO_JSON_JOLT_RESPONSE,
    };

    use super::JoltTransformLayer;

    impl TryFrom<&ServiceConfig> for JoltTransformLayer {
        type Error = MarsError;

        fn try_from(value: &ServiceConfig) -> Result<Self, Self::Error> {
            let request_transform = value
                .params
                .get_value(TRANSFORM_JSON_TO_JSON_JOLT_REQUEST)
                .cloned();
            let response_transform = value
                .params
                .get_value(TRANSFORM_JSON_TO_JSON_JOLT_RESPONSE)
                .cloned();
            let request_transform = match request_transform {
                Some(reqquest_transform) => match serde_json::from_value(reqquest_transform) {
                    Ok(request_transform) => request_transform,
                    Err(error) => {
                        return Err(MarsError::ServiceConfigError(format!(
                            "unable to parse transform spec, failed with error {error}"
                        )))
                    }
                },
                None => None,
            };
            let response_transform = match response_transform {
                Some(response_transform) => match serde_json::from_value(response_transform) {
                    Ok(response_transform) => response_transform,
                    Err(error) => {
                        return Err(MarsError::ServiceConfigError(format!(
                            "unable to parse transform spec, failed with error {error}"
                        )))
                    }
                },
                None => None,
            };
            if request_transform.is_none() && response_transform.is_none() {
                return Err(MarsError::ServiceConfigError(
                    "both request_transform and response_transform failed not  avaiabile".into(),
                ));
            }
            Ok(JoltTransformLayer {
                response_transform,
                request_transform,
            })
        }
    }
}
