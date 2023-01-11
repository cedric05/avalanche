use std::{convert::TryFrom, future::Future, pin::Pin};

use http::{Request, Response};
use hyper::{client::HttpConnector, Client};
use hyper_tls::HttpsConnector;
use native_tls::{Identity, TlsConnector};
use tokio_native_tls::TlsConnector as TokioNativeTlsConnector;

use tower::{Layer, Service};

use crate::{config::ServiceConfig, error::MarsError};

#[derive(Clone)]
pub(crate) struct SslAuth<S> {
    inner: S,
}

struct SslAuthLayer;

impl<S> Layer<S> for SslAuthLayer {
    type Service = SslAuth<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SslAuth { inner }
    }
}

impl TryFrom<&ServiceConfig> for Identity {
    type Error = MarsError;

    fn try_from(value: &ServiceConfig) -> Result<Self, Self::Error> {
        let pkcs_der = value.get_authparam_value_as_str("pkcs12")?;
        let password = value.get_authparam_value_as_str("pkcs12_password")?;
        let pkcs_der = base64::decode(pkcs_der).map_err(|err| {
            MarsError::ServiceConfigError(format!("unable to parse pkcs_der: {}", err))
        })?;
        let identity = Identity::from_pkcs12(&pkcs_der, password).map_err(|err| {
            MarsError::ServiceConfigError(format!("unable to parse pkcs12 error: {}", err))
        })?;
        Ok(identity)
    }
}

impl<ReqBody, ResBody, S> Service<Request<ReqBody>> for SslAuth<S>
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
        let fut = self.inner.call(req);
        Box::pin(async { fut.await })
    }
}

pub(crate) fn ssl_auth_client_from_service_config(
    value: &ServiceConfig,
) -> Result<Client<HttpsConnector<HttpConnector>>, MarsError> {
    let identity = Identity::try_from(value)?;
    let native_tls_connector =
        TlsConnector::builder()
            .identity(identity)
            .build()
            .map_err(|err| {
                MarsError::ServiceConfigError(format!("tlsbuild failed into error: {}", err))
            })?;
    let tokio_native_tls_connector = TokioNativeTlsConnector::from(native_tls_connector);
    let mut http_connector = HttpConnector::new();
    http_connector.enforce_http(false);
    let https: HttpsConnector<HttpConnector> =
        HttpsConnector::from((http_connector, tokio_native_tls_connector));
    let client = Client::builder().build::<_, hyper::Body>(https);
    Ok(client)
}

#[cfg(test)]
mod test {
    use std::{error::Error, io::Read};

    use http::Request;
    use hyper::{client::HttpConnector, Body, Client};
    use hyper_tls::HttpsConnector;
    use native_tls::{Identity, TlsConnector};
    use tokio_native_tls::TlsConnector as TokioNativeTlsConnector;

    const BAD_SSL_PASSWORD: &str = "badssl.com";
    const CERTIFICATE_P12: &str = "./resources/badssl.com-client.p12";

    #[tokio::test]
    pub(crate) async fn test_certificate() -> Result<(), Box<dyn Error>> {
        let mut file = std::fs::File::open(CERTIFICATE_P12)?;
        let mut certificate_der = vec![];
        file.read_to_end(&mut certificate_der)?;

        let identity = Identity::from_pkcs12(&certificate_der, BAD_SSL_PASSWORD)?;
        let native_tls_connector = TlsConnector::builder().identity(identity).build()?;
        let tokio_native_tls_connector = TokioNativeTlsConnector::from(native_tls_connector);
        let mut http_connector = HttpConnector::new();
        http_connector.enforce_http(false);
        let connector: HttpsConnector<HttpConnector> =
            HttpsConnector::from((http_connector, tokio_native_tls_connector));
        let client = Client::builder().build::<_, hyper::Body>(connector);
        let req = Request::builder()
            .uri("https://client.badssl.com/")
            .body(Body::empty())?;
        let response = client.request(req).await?;
        for header in response.headers() {
            println!("header (key: {:?}, value: {:?})", header.0, header.1);
        }
        assert_eq!(response.status(), 200);
        Ok(())
    }
}
