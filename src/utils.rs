#[macro_export]
macro_rules! impl_proxy_service {
    ($service:ident) => {
        #[async_trait::async_trait]
        impl $crate::project::ProxyService
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
