use std::future::Future;
use std::pin::Pin;
use std::time::SystemTime;

use aws_sigv4::http_request::{sign, PayloadChecksumKind, SignableBody, SignableRequest};
use aws_sigv4::{http_request::SigningSettings, signing_params::Builder as SignparamsBuilder};

use http::{Request, Response};

use hyper::body;

use serde::{Deserialize, Serialize};
use tower::{Layer, Service};

use crate::config::ServiceConfig;
use crate::error::MarsError;

#[derive(Clone)]
pub(crate) struct AwsAuth<S> {
    access_key: String,
    secret_key: String,
    region: String,
    service_name: String,
    sign_content: bool,
    inner: S,
}

pub(crate) struct AwsAuthLayer {
    access_key: String,
    secret_key: String,
    region: String,
    service_name: String,
    sign_content: bool,
}

impl<S> Layer<S> for AwsAuthLayer {
    type Service = AwsAuth<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AwsAuth {
            access_key: self.access_key.clone(),
            secret_key: self.secret_key.clone(),
            region: self.region.clone(),
            service_name: self.service_name.clone(),
            sign_content: self.sign_content.clone(),
            inner,
        }
    }
}

type ResBody = hyper::Body;
type ReqBody = hyper::Body;
type HyperError = hyper::Error;

impl<S> Service<Request<ReqBody>> for AwsAuth<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>, Error = HyperError>
        + Send
        + 'static
        + Clone,
    S::Future: 'static,
    <S as Service<Request<ReqBody>>>::Future: Send,
{
    type Response = Response<ResBody>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response<ResBody>, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let access_key = self.access_key.clone();
        let secret_key = self.secret_key.clone();
        let region = self.region.clone();
        let service_name = self.service_name.clone();
        let (parts, body) = req.into_parts();
        let sign_content = self.sign_content.clone();
        let mut orig = self.inner.clone();
        Box::pin(async move {
            let mut settings = SigningSettings::default();
            if sign_content {
                settings.payload_checksum_kind = PayloadChecksumKind::XAmzSha256
            } else {
                settings.payload_checksum_kind = PayloadChecksumKind::NoHeader
            };
            let body = body::to_bytes(body).await?;
            let sign_body = SignableBody::Bytes(&body);
            let params = SignparamsBuilder::default();
            let params = params
                .access_key(&access_key)
                .secret_key(&secret_key)
                .region(&region)
                .service_name(&service_name)
                .time(SystemTime::now())
                .settings(settings)
                .build()
                .unwrap();
            let signable =
                SignableRequest::new(&parts.method, &parts.uri, &parts.headers, sign_body);
            let out = sign(signable, &params).unwrap();
            let (output, _signature) = out.into_parts();
            let mut signable = Request::from_parts(parts, hyper::Body::from(body));
            output.apply_to_request(&mut signable);
            orig.call(signable).await
        })
    }
}

#[derive(Serialize, Deserialize)]
struct AwsAuthParams {
    access_key: String,
    secret_key: String,
    region: String,
    service: String,
    #[serde(default)]
    sign_content: bool,
}

impl TryFrom<&ServiceConfig> for AwsAuthLayer {
    type Error = MarsError;

    fn try_from(value: &ServiceConfig) -> Result<Self, Self::Error> {
        let aws_auth_params: AwsAuthParams = serde_json::from_value(value.auth.get_params())
            .map_err(|err| {
                MarsError::ServiceConfigError(format!(
                    "unable to parse auth params for aws auth configuration error:{}",
                    err
                ))
            })?;
        let aws_auth_layer = AwsAuthLayer {
            access_key: aws_auth_params.access_key,
            secret_key: aws_auth_params.secret_key,
            region: aws_auth_params.region,
            service_name: aws_auth_params.service,
            sign_content: aws_auth_params.sign_content,
        };
        Ok(aws_auth_layer)
    }
}

#[cfg(test)]
mod test {
    use std::{borrow::Cow, error::Error, fmt, time::SystemTime};

    use aws_sigv4::http_request::{sign, SignableRequest, SigningSettings};
    use aws_sigv4::signing_params::Builder as SignparamsBuilder;
    use http::{HeaderValue, Request};
    use time::{format_description, PrimitiveDateTime};

    fn haha() {
        const DATE_TIME_FORMAT: &str = "[year][month][day]T[hour][minute][second]Z";

        #[derive(Debug)]
        pub(crate) struct ParseError(Cow<'static, str>);

        impl fmt::Display for ParseError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "failed to parse time: {}", self.0)
            }
        }

        impl Error for ParseError {}

        pub(crate) fn parse_date_time(date_time_str: &str) -> Result<SystemTime, ParseError> {
            let date_time = PrimitiveDateTime::parse(
                date_time_str,
                &format_description::parse(DATE_TIME_FORMAT).unwrap(),
            )
            .map_err(|err| ParseError(err.to_string().into()))?
            .assume_utc();
            Ok(date_time.into())
        }

        let settings = SigningSettings::default();
        let params: SignparamsBuilder<SigningSettings> = Default::default();
        let params = params
            .access_key("AKIDEXAMPLE")
            .secret_key("wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY")
            .region("us-east-1")
            .service_name("service")
            .time(parse_date_time("20150830T123600Z").unwrap())
            .settings(settings)
            .build()
            .unwrap();

        let original = Request::builder()
            .uri("https://service.us-east-1.amazonaws.com")
            .header("some-header", HeaderValue::from_str("テスト").unwrap())
            .body("")
            .unwrap();
        let signable = SignableRequest::from(&original);
        let out = sign(signable, &params).unwrap();
        let (output, signature) = out.into_parts();
        let mut signed = original;
        output.apply_to_request(&mut signed);

        assert_eq!(
            "52b637b2211de99f32ec24cbf1dd5bc0cad970b4a9e1dc6927e158cfb2f47bbe",
            signature
        );

        let expected = http::Request::builder()
            .uri("https://some-endpoint.some-region.amazonaws.com")
            .header("some-header", HeaderValue::from_str("テスト").unwrap())
            .header(
                "x-amz-date",
                HeaderValue::from_str("20150830T123600Z").unwrap(),
            )
            .header(
                "authorization",
                HeaderValue::from_str(
                    "AWS4-HMAC-SHA256 \
                        Credential=AKIDEXAMPLE/20150830/us-east-1/service/aws4_request, \
                        SignedHeaders=host;some-header;x-amz-date, \
                        Signature=4596b207a7fc6bdf18725369bc0cd7022cf20efbd2c19730549f42d1a403648e",
                )
                .unwrap(),
            )
            .body(hyper::Body::empty())
            .unwrap();
        println!("expected is {:?}", expected);
        println!("actual is {:?}", signed);
    }

    #[test]
    fn haha2() {
        haha()
    }
}
