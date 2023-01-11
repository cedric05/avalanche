use std::error::Error;
use std::time::Duration;

use http::{Request, Response};
use hyper::client::HttpConnector;
use hyper::Body;
use hyper_tls::HttpsConnector;

use tower_boxed_service_sync::BoxCloneSyncService;

use crate::common::CommonUpdateQueryNHeaderLayer;
use mars_config::{AuthType, MarsError};
use tower::limit::ConcurrencyLimitLayer;
use tower::timeout::TimeoutLayer;
use tower::ServiceBuilder;

use mars_config::ServiceConfig;

#[cfg(feature = "awsauth")]
use crate::awsauth;
#[cfg(feature = "basicauth")]
use crate::basicauth;

#[cfg(feature = "digestauth")]
use crate::digestauth;
#[cfg(feature = "hawkauth")]
use crate::hawkauth;
use crate::headerauth;
#[cfg(feature = "x509auth")]
use crate::x509;

pub(crate) fn simple_hyper_https_client() -> hyper::Client<hyper_tls::HttpsConnector<HttpConnector>>
{
    hyper::Client::builder().build::<_, hyper::Body>(HttpsConnector::new())
}

pub type ProxyService =
    BoxCloneSyncService<Request<Body>, Response<Body>, Box<dyn Error + Send + Sync>>;

pub fn get_auth_service(
    service_config: ServiceConfig,
) -> Result<ProxyService, mars_config::MarsError> {
    let timeout = service_config
        .get_timeout()
        .map(|x| Duration::from_secs(x as u64))
        .map(TimeoutLayer::new);
    let concurrency_limit = service_config
        .get_concurrency_timeout()
        .map(|x| x as usize)
        .map(ConcurrencyLimitLayer::new);
    match service_config.auth.auth_type() {
        #[cfg(feature = "basicauth")]
        AuthType::BasicAuth => Ok(ServiceBuilder::new()
            .layer(BoxCloneSyncService::layer())
            .option_layer(timeout)
            .option_layer(concurrency_limit)
            .layer(CommonUpdateQueryNHeaderLayer::new(service_config.clone()))
            .layer(basicauth::BasicAuthLayer::try_from(&service_config).unwrap())
            .service(simple_hyper_https_client())),
        AuthType::HeaderAuth => Ok(ServiceBuilder::new()
            .layer(BoxCloneSyncService::layer())
            .option_layer(timeout)
            .option_layer(concurrency_limit)
            .layer(CommonUpdateQueryNHeaderLayer::new(service_config.clone()))
            .layer(headerauth::HeaderAuthLayer::try_from(&service_config)?)
            .service(simple_hyper_https_client())),
        #[cfg(feature = "awsauth")]
        AuthType::AwsAuth => Ok(ServiceBuilder::new()
            .layer(BoxCloneSyncService::layer())
            .option_layer(timeout)
            .option_layer(concurrency_limit)
            .layer(CommonUpdateQueryNHeaderLayer::new(service_config.clone()))
            .layer(awsauth::AwsAuthLayer::try_from(&service_config)?)
            .service(simple_hyper_https_client())),
        #[cfg(feature = "x509auth")]
        AuthType::X509Auth => Ok(ServiceBuilder::new()
            .layer(BoxCloneSyncService::layer())
            .option_layer(timeout)
            .option_layer(concurrency_limit)
            .layer(CommonUpdateQueryNHeaderLayer::new(service_config.clone()))
            .service(x509::service_config::ssl_auth_client_from_service_config(
                &service_config,
            )?)),
        #[cfg(feature = "hawkauth")]
        AuthType::HawkAuth => Ok(ServiceBuilder::new()
            .layer(BoxCloneSyncService::layer())
            .option_layer(timeout)
            .option_layer(concurrency_limit)
            .layer(CommonUpdateQueryNHeaderLayer::new(service_config.clone()))
            .layer(hawkauth::HawkAuthLayer::try_from(&service_config)?)
            .service(simple_hyper_https_client())),
        #[cfg(feature = "digestauth")]
        AuthType::DigestAuth => Ok(ServiceBuilder::new()
            .layer(BoxCloneSyncService::layer())
            .option_layer(timeout)
            .option_layer(concurrency_limit)
            .layer(CommonUpdateQueryNHeaderLayer::new(service_config.clone()))
            .layer(digestauth::DigestAuthLayer::try_from(&service_config)?)
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
