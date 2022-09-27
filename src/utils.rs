#[cfg(feature = "awsauth")]
use crate::awsauth::AwsAuth;
#[cfg(feature = "basicauth")]
use crate::basicauth::BasicAuth;
use crate::config::ServiceConfig;
#[cfg(feature = "digestauth")]
use crate::digestauth::DigestAuth;
#[cfg(feature = "hawkauth")]
use crate::hawkauth::HawkAuth;
use crate::headerauth::HeaderAuth;
use crate::noauth::NoAuth;
use crate::project::ProxyService;
#[cfg(feature = "x509auth")]
use crate::x509::SslAuth;
use tower::Service;

macro_rules! impl_proxy_service {
    ($service:ident) => {
        #[async_trait::async_trait]
        impl ProxyService
            for $service<hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>>
        {
            async fn handle_service(
                &mut self,
                url: &str,
                service_config: &ServiceConfig,
                mut request: hyper::Request<hyper::Body>,
            ) -> Result<http::Response<hyper::Body>, Box<dyn std::error::Error>> {
                service_config.get_updated_request(url, &mut request)?;
                let response = self.call(request).await?;
                Ok(response)
            }
        }
    };
}

#[cfg(feature = "awsauth")]
impl_proxy_service!(AwsAuth);
#[cfg(feature = "basicauth")]
impl_proxy_service!(BasicAuth);
#[cfg(feature = "digestauth")]
impl_proxy_service!(DigestAuth);
#[cfg(feature = "hawkauth")]
impl_proxy_service!(HawkAuth);
#[cfg(feature = "x509auth")]
impl_proxy_service!(SslAuth);

impl_proxy_service!(HeaderAuth);
impl_proxy_service!(NoAuth);
