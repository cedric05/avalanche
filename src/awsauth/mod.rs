use std::future::Future;
use std::pin::Pin;
use std::time::SystemTime;

use aws_sigv4::http_request::{sign, SignableBody, SignableRequest};
use aws_sigv4::{http_request::SigningSettings, signing_params::Builder as SignparamsBuilder};

use http::{Request, Response};
use hyper::client::HttpConnector;

use hyper::{body, Client};
use hyper_tls::HttpsConnector;

use tower::{Layer, Service, ServiceBuilder};

use crate::config::ServiceConfig;
use crate::error::MarsError;
use crate::impl_proxy_service;

#[derive(Clone)]
pub struct AwsAuth<S> {
    access_key: String,
    secret_key: String,
    region: String,
    service_name: String,
    inner: S,
}

pub struct AwsAuthLayer {
    access_key: String,
    secret_key: String,
    region: String,
    service_name: String,
}

impl<S> Layer<S> for AwsAuthLayer {
    type Service = AwsAuth<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AwsAuth {
            access_key: self.access_key.clone(),
            secret_key: self.secret_key.clone(),
            region: self.region.clone(),
            service_name: self.service_name.clone(),
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
        let mut orig = self.inner.clone();
        Box::pin(async move {
            let settings = SigningSettings::default();
            let params: SignparamsBuilder<SigningSettings> = Default::default();
            let params = params
                .access_key(&access_key)
                .secret_key(&secret_key)
                .region(&region)
                .service_name(&service_name)
                .time(SystemTime::now())
                .settings(settings)
                .build()
                .unwrap();
            let body = body::to_bytes(body).await?;
            let signable = SignableRequest::new(
                &parts.method,
                &parts.uri,
                &parts.headers,
                // TODO bytable request is not working
                SignableBody::Bytes(&body),
            );
            let out = sign(signable, &params).unwrap();
            let (output, _signature) = out.into_parts();
            let body = hyper::Body::from(body);
            let mut signable = Request::from_parts(parts, body);
            output.apply_to_request(&mut signable);
            orig.call(signable).await
        })
    }
}

impl TryFrom<&ServiceConfig> for AwsAuthLayer {
    type Error = MarsError;

    fn try_from(value: &ServiceConfig) -> Result<Self, Self::Error> {
        let access_key = value.get_handler_config("access_key")?;
        let secret_key = value.get_handler_config("secret_key")?;
        let region = value.get_handler_config("region")?;
        let service = value.get_handler_config("service")?;
        let aws_auth_layer = AwsAuthLayer {
            access_key: access_key.to_string(),
            secret_key: secret_key.to_string(),
            region: region.to_string(),
            service_name: service.to_string(),
        };
        Ok(aws_auth_layer)
    }
}

impl TryFrom<&ServiceConfig> for AwsAuth<Client<HttpsConnector<HttpConnector>>> {
    type Error = MarsError;

    fn try_from(value: &ServiceConfig) -> Result<Self, Self::Error> {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);
        let auth_layer = AwsAuthLayer::try_from(value)?;
        let res = ServiceBuilder::new().layer(auth_layer).service(client);
        Ok(res)
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

impl_proxy_service!(AwsAuth);
