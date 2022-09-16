use crate::awsauth::AwsAuth;
use crate::basicauth::BasicAuth;
use crate::config::ServiceConfig;
use crate::hawkauth::HawkAuth;
use crate::headerauth::HeaderAuth;
use crate::project::ProxyService;
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

impl_proxy_service!(HeaderAuth);
impl_proxy_service!(SslAuth);
impl_proxy_service!(BasicAuth);
impl_proxy_service!(AwsAuth);
impl_proxy_service!(HawkAuth);
