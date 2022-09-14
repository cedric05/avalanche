use std::{convert::TryFrom, error::Error, future::Future, pin::Pin};

use async_trait::async_trait;
use http::{Request, Response};
use hyper::{client::HttpConnector, Body, Client};
use hyper_tls::HttpsConnector;
use native_tls::{Identity, TlsConnector};
use tokio_native_tls::TlsConnector as TokioNativeTlsConnector;

use tower::{Layer, Service, ServiceBuilder};

use crate::{config::ServiceConfig, error::MarsError, project::ProxyService};

#[derive(Clone)]
pub struct SslAuth<S> {
    inner: S,
}

struct SslAuthLayer;

impl<S> Layer<S> for SslAuthLayer {
    type Service = SslAuth<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SslAuth { inner: inner }
    }
}

impl TryFrom<&ServiceConfig> for Identity {
    type Error = MarsError;

    fn try_from(value: &ServiceConfig) -> Result<Self, Self::Error> {
        let pkcs_der = value
            .handler
            .params
            .get("pkcs12")
            .ok_or(MarsError::ServiceConfigError)?
            .as_str()
            .ok_or(MarsError::ServiceConfigError)?;
        let password = value
            .handler
            .params
            .get("pkcs12_password")
            .ok_or(MarsError::ServiceConfigError)?
            .as_str()
            .ok_or(MarsError::ServiceConfigError)?;
        let pkcs_der = base64::decode(pkcs_der).map_err(|_| MarsError::ServiceConfigError)?;
        let identity = Identity::from_pkcs12(&pkcs_der, password)
            .map_err(|_| MarsError::ServiceConfigError)?;
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

#[async_trait]
impl ProxyService for SslAuth<hyper::Client<HttpsConnector<HttpConnector>>> {
    async fn handle_service(
        &mut self,
        url: &str,
        service_config: &ServiceConfig,
        request: hyper::Request<Body>,
    ) -> Result<Response<Body>, Box<dyn Error>> {
        let mut request = request;
        service_config.get_updated_request(url, &mut request)?;
        let response = self.call(request).await?;
        Ok(response)
    }
}

impl TryFrom<&ServiceConfig> for SslAuth<Client<HttpsConnector<HttpConnector>>> {
    type Error = MarsError;

    fn try_from(value: &ServiceConfig) -> Result<Self, Self::Error> {
        let identity = Identity::try_from(value)?;
        let native_tls_connector = TlsConnector::builder()
            .identity(identity)
            .build()
            .map_err(|_| MarsError::ServiceConfigError)?;
        let tokio_native_tls_connector = TokioNativeTlsConnector::from(native_tls_connector);
        let mut http_connector = HttpConnector::new();
        http_connector.enforce_http(false);
        let https: HttpsConnector<HttpConnector> =
            HttpsConnector::from((http_connector, tokio_native_tls_connector));
        let client = Client::builder().build::<_, hyper::Body>(https);
        let service = ServiceBuilder::new().layer(SslAuthLayer).service(client);
        Ok(service)
    }
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
    pub async fn test_certificate() -> Result<(), Box<dyn Error>> {
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
