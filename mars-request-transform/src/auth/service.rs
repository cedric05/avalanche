//! This module contains the implementation of the authentication service.
//!
//! The `get_auth_service` function is the entry point for obtaining an authentication service based on the provided `ServiceConfig`.
//! It returns a `Result` containing a `ProxyService` or a `MarsError` if the authentication type is not supported.
//!
//! The `simple_hyper_https_client` function creates a simple HTTPS client using the `hyper` and `hyper_tls` crates.
//!
//! The `ProxyService` type is an alias for a boxed cloneable synchronous service that handles HTTP requests and responses.
//!
//! The `get_auth_service` function takes a `ServiceConfig` as input and returns a `Result` containing a `ProxyService` or a `MarsError`.
//! It builds the authentication service based on the authentication type specified in the `ServiceConfig`.
//! The authentication service is constructed using the `ServiceBuilder` from the `tower` crate and various layers and options.
//! The layers include the `BoxCloneSyncService` layer, the `TimeoutLayer`, the `ConcurrencyLimitLayer`, and the `CommonUpdateQueryNHeaderLayer`.
//! The options include the transformation layers for Jolt, XML, YAML, and JSON, based on the features enabled.
//! The authentication layer is added based on the authentication type specified in the `ServiceConfig`.
//! Finally, the authentication service is wrapped with the `simple_hyper_https_client` function to create the final service.
//!
//! The authentication types supported are:
//! - BasicAuth
//! - HeaderAuth
//! - AwsAuth (requires the `awsauth` feature)
//! - X509Auth (requires the `x509auth` feature)
//! - HawkAuth (requires the `hawkauth` feature)
//! - DigestAuth (requires the `digestauth` feature)
//! - NoAuth
//!
//! If the authentication type is not supported or not registered, an error of type `MarsError::ServiceNotRegistered` is returned.
//!
//! Example usage:
//!
//! ```rust
//! use mars_config::ServiceConfig;
//! use mars_rover::auth::get_auth_service;
//!
//! let service_config = ServiceConfig::new();
//! let auth_service = get_auth_service(service_config);
//!
//! match auth_service {
//!     Ok(service) => {
//!         // Use the authentication service
//!     },
//!     Err(error) => {
//!         // Handle the error
//!     }
//! }
//! ```
//!
//! Note: This documentation is auto-generated and may not be up-to-date. Please refer to the source code for the latest documentation.
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
    let (
        jolt_transform_layer,
        xml_transform_layer,
        yaml_transform_layer,
        yaml_to_json_trasnsform_layer,
    ) = if cfg!(feature = "transform") {
        let jolt_transform_layer = if service_config
            .get_params(crate::transform::jolt_service_config::TRANSFORM_JSON_TO_JSON_JOLT_REQUEST)
            .is_some()
            || service_config
                .get_params(
                    crate::transform::jolt_service_config::TRANSFORM_JSON_TO_JSON_JOLT_RESPONSE,
                )
                .is_some()
        {
            Some(crate::transform::JoltTransformLayer::try_from(
                &service_config,
            )?)
        } else {
            None
        };
        let xml_transform_layer = if service_config
            .get_params(crate::transform::xml_service_config::TRANSFORM_XML_TO_JSON_RESPONSE)
            .is_some()
            || service_config
                .get_params(crate::transform::xml_service_config::TRANSFORM_XML_JSON_REQUEST)
                .is_some()
        {
            Some(crate::transform::XmlTransformJsonLayer::try_from(
                &service_config,
            )?)
        } else {
            None
        };
        let yaml_transform_layer = if service_config
            .get_params(crate::transform::yaml_service_config::TRANSFORM_YAML_JSON_REQUEST)
            .is_some()
            || service_config
                .get_params(crate::transform::yaml_service_config::TRANSFORM_YAML_TO_JSON_RESPONSE)
                .is_some()
        {
            Some(crate::transform::YamlTransformJsonLayer::try_from(
                &service_config,
            )?)
        } else {
            None
        };
        let json_to_yaml_transform_layer = if service_config
            .get_params(crate::transform::json_to_yaml_service_config::TRANSFORM_JSON_YAML_REQUEST)
            .is_some()
            || service_config
                .get_params(
                    crate::transform::json_to_yaml_service_config::TRANSFORM_JSON_TO_YAML_RESPONSE,
                )
                .is_some()
        {
            Some(crate::transform::JsonTransformYamlLayer::try_from(
                &service_config,
            )?)
        } else {
            None
        };
        (
            jolt_transform_layer,
            xml_transform_layer,
            yaml_transform_layer,
            json_to_yaml_transform_layer,
        )
    } else {
        (None, None, None, None)
    };
    match service_config.auth.auth_type() {
        #[cfg(feature = "basicauth")]
        AuthType::BasicAuth => Ok(ServiceBuilder::new()
            .layer(BoxCloneSyncService::layer())
            .option_layer(timeout)
            .option_layer(concurrency_limit)
            .layer(CommonUpdateQueryNHeaderLayer::new(service_config.clone()))
            .option_layer(xml_transform_layer)
            .option_layer(jolt_transform_layer)
            .option_layer(yaml_transform_layer)
            .option_layer(yaml_to_json_trasnsform_layer)
            .layer(basicauth::BasicAuthLayer::try_from(&service_config).unwrap())
            .service(simple_hyper_https_client())),
        AuthType::HeaderAuth => Ok(ServiceBuilder::new()
            .layer(BoxCloneSyncService::layer())
            .option_layer(timeout)
            .option_layer(concurrency_limit)
            .layer(CommonUpdateQueryNHeaderLayer::new(service_config.clone()))
            .option_layer(xml_transform_layer)
            .option_layer(jolt_transform_layer)
            .option_layer(yaml_transform_layer)
            .option_layer(yaml_to_json_trasnsform_layer)
            .layer(headerauth::HeaderAuthLayer::try_from(&service_config)?)
            .service(simple_hyper_https_client())),
        #[cfg(feature = "awsauth")]
        AuthType::AwsAuth => Ok(ServiceBuilder::new()
            .layer(BoxCloneSyncService::layer())
            .option_layer(timeout)
            .option_layer(concurrency_limit)
            .layer(CommonUpdateQueryNHeaderLayer::new(service_config.clone()))
            .option_layer(xml_transform_layer)
            .option_layer(jolt_transform_layer)
            .option_layer(yaml_transform_layer)
            .option_layer(yaml_to_json_trasnsform_layer)
            .layer(awsauth::AwsAuthLayer::try_from(&service_config)?)
            .service(simple_hyper_https_client())),
        #[cfg(feature = "x509auth")]
        AuthType::X509Auth => Ok(ServiceBuilder::new()
            .layer(BoxCloneSyncService::layer())
            .option_layer(timeout)
            .option_layer(concurrency_limit)
            .layer(CommonUpdateQueryNHeaderLayer::new(service_config.clone()))
            .option_layer(xml_transform_layer)
            .option_layer(jolt_transform_layer)
            .option_layer(yaml_transform_layer)
            .option_layer(yaml_to_json_trasnsform_layer)
            .service(x509::service_config::ssl_auth_client_from_service_config(
                &service_config,
            )?)),
        #[cfg(feature = "hawkauth")]
        AuthType::HawkAuth => Ok(ServiceBuilder::new()
            .layer(BoxCloneSyncService::layer())
            .option_layer(timeout)
            .option_layer(concurrency_limit)
            .layer(CommonUpdateQueryNHeaderLayer::new(service_config.clone()))
            .option_layer(xml_transform_layer)
            .option_layer(jolt_transform_layer)
            .option_layer(yaml_transform_layer)
            .option_layer(yaml_to_json_trasnsform_layer)
            .layer(hawkauth::HawkAuthLayer::try_from(&service_config)?)
            .service(simple_hyper_https_client())),
        #[cfg(feature = "digestauth")]
        AuthType::DigestAuth => Ok(ServiceBuilder::new()
            .layer(BoxCloneSyncService::layer())
            .option_layer(timeout)
            .option_layer(concurrency_limit)
            .layer(CommonUpdateQueryNHeaderLayer::new(service_config.clone()))
            .option_layer(xml_transform_layer)
            .option_layer(jolt_transform_layer)
            .option_layer(yaml_transform_layer)
            .option_layer(yaml_to_json_trasnsform_layer)
            .layer(digestauth::DigestAuthLayer::try_from(&service_config)?)
            .service(simple_hyper_https_client())),
        AuthType::NoAuth => Ok(ServiceBuilder::new()
            .layer(BoxCloneSyncService::layer())
            .option_layer(timeout)
            .option_layer(concurrency_limit)
            .layer(CommonUpdateQueryNHeaderLayer::new(service_config.clone()))
            .option_layer(xml_transform_layer)
            .option_layer(jolt_transform_layer)
            .option_layer(yaml_transform_layer)
            .option_layer(yaml_to_json_trasnsform_layer)
            .service(simple_hyper_https_client())),
        _ => Err(MarsError::ServiceNotRegistered),
    }
}
