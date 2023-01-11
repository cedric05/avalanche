use std::error::Error;
use std::time::Duration;

use http::{Request, Response};
use hyper::client::HttpConnector;
use hyper::Body;
use hyper_tls::HttpsConnector;
use mars_config::AuthType;
use tower::limit::ConcurrencyLimitLayer;
use tower::timeout::TimeoutLayer;
use tower::ServiceBuilder;
use tower_boxed_service_sync::BoxCloneSyncService;

use crate::config::CommonUpdateQueryNHeaderLayer;
use crate::headerauth::HeaderAuthLayer;
use crate::{config::ServiceConfig, error::MarsError};

#[cfg(feature = "awsauth")]
use crate::awsauth::AwsAuthLayer;
#[cfg(feature = "digestauth")]
use crate::digestauth::DigestAuthLayer;
#[cfg(feature = "hawkauth")]
use crate::hawkauth::HawkAuthLayer;
#[cfg(feature = "x509auth")]
use crate::x509::ssl_auth_client_from_service_config;

#[cfg(feature = "basicauth")]
use crate::basicauth::BasicAuthLayer;

fn simple_hyper_https_client() -> hyper::Client<hyper_tls::HttpsConnector<HttpConnector>> {
    hyper::Client::builder().build::<_, hyper::Body>(HttpsConnector::new())
}

pub type ProxyService =
    BoxCloneSyncService<Request<Body>, Response<Body>, Box<dyn Error + Send + Sync>>;

pub(crate) fn get_auth_service(service_config: ServiceConfig) -> Result<ProxyService, MarsError> {
    let timeout = service_config
        .get_timeout()
        .map(|x| Duration::from_secs(x as u64))
        .map(TimeoutLayer::new);
    let concurrency_limit = service_config
        .get_concurrency_timeout()
        .map(|x| x as usize)
        .map(ConcurrencyLimitLayer::new);
    match service_config.auth.auth_type {
        #[cfg(feature = "basicauth")]
        AuthType::BasicAuth => Ok(ServiceBuilder::new()
            .layer(BoxCloneSyncService::layer())
            .option_layer(timeout)
            .option_layer(concurrency_limit)
            .layer(CommonUpdateQueryNHeaderLayer::new(service_config.clone()))
            .layer(BasicAuthLayer::try_from(&service_config)?)
            .service(simple_hyper_https_client())),
        AuthType::HeaderAuth => Ok(ServiceBuilder::new()
            .layer(BoxCloneSyncService::layer())
            .option_layer(timeout)
            .option_layer(concurrency_limit)
            .layer(CommonUpdateQueryNHeaderLayer::new(service_config.clone()))
            .layer(HeaderAuthLayer::try_from(&service_config)?)
            .service(simple_hyper_https_client())),
        #[cfg(feature = "awsauth")]
        AuthType::AwsAuth => Ok(ServiceBuilder::new()
            .layer(BoxCloneSyncService::layer())
            .option_layer(timeout)
            .option_layer(concurrency_limit)
            .layer(CommonUpdateQueryNHeaderLayer::new(service_config.clone()))
            .layer(AwsAuthLayer::try_from(&service_config)?)
            .service(simple_hyper_https_client())),
        #[cfg(feature = "x509auth")]
        AuthType::X509Auth => Ok(ServiceBuilder::new()
            .layer(BoxCloneSyncService::layer())
            .option_layer(timeout)
            .option_layer(concurrency_limit)
            .layer(CommonUpdateQueryNHeaderLayer::new(service_config.clone()))
            .service(ssl_auth_client_from_service_config(&service_config)?)),
        #[cfg(feature = "hawkauth")]
        AuthType::HawkAuth => Ok(ServiceBuilder::new()
            .layer(BoxCloneSyncService::layer())
            .option_layer(timeout)
            .option_layer(concurrency_limit)
            .layer(CommonUpdateQueryNHeaderLayer::new(service_config.clone()))
            .layer(HawkAuthLayer::try_from(&service_config)?)
            .service(simple_hyper_https_client())),
        #[cfg(feature = "digestauth")]
        AuthType::DigestAuth => Ok(ServiceBuilder::new()
            .layer(BoxCloneSyncService::layer())
            .option_layer(timeout)
            .option_layer(concurrency_limit)
            .layer(CommonUpdateQueryNHeaderLayer::new(service_config.clone()))
            .layer(DigestAuthLayer::try_from(&service_config)?)
            .service(simple_hyper_https_client())),
        AuthType::NoAuth => Ok(ServiceBuilder::new()
            .layer(BoxCloneSyncService::layer())
            .option_layer(timeout)
            .option_layer(concurrency_limit)
            .layer(CommonUpdateQueryNHeaderLayer::new(service_config.clone()))
            .service(simple_hyper_https_client())),
        _ => Err(MarsError::ServiceNotRegistered),
    }
}
