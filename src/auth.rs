use hyper::client::HttpConnector;
use hyper::Client;
use hyper_tls::HttpsConnector;

use crate::headerauth::HeaderAuth;
use crate::noauth::NoAuth;
use crate::{config::ServiceConfig, error::MarsError, project::ProxyService};

#[cfg(feature = "awsauth")]
use crate::awsauth::AwsAuth;
#[cfg(feature = "digestauth")]
use crate::digestauth::DigestAuth;
#[cfg(feature = "hawkauth")]
use crate::hawkauth::HawkAuth;
#[cfg(feature = "x509auth")]
use crate::x509::SslAuth;

#[cfg(feature = "basicauth")]
use crate::basicauth::BasicAuth;

pub (crate) fn get_auth_service(
    service_config: ServiceConfig,
) -> Result<(ServiceConfig, Box<dyn ProxyService>), MarsError> {
    match service_config.handler.handler_type.as_str() {
        #[cfg(feature = "basicauth")]
        "basic_auth" => {
            let basic_auth_service =
                BasicAuth::<Client<HttpsConnector<HttpConnector>>>::try_from(&service_config)?;
            let basic_auth_config: (ServiceConfig, Box<dyn ProxyService>) =
                (service_config, Box::new(basic_auth_service));
            Ok(basic_auth_config)
        }
        "header_auth" => {
            let header_auth_service =
                HeaderAuth::<Client<HttpsConnector<HttpConnector>>>::try_from(&service_config)?;
            let header_auth_config: (ServiceConfig, Box<dyn ProxyService>) =
                (service_config, Box::new(header_auth_service));
            Ok(header_auth_config)
        }
        #[cfg(feature = "awsauth")]
        "aws_auth" => {
            let aws_auth_service =
                AwsAuth::<Client<HttpsConnector<HttpConnector>>>::try_from(&service_config)?;
            let aws_auth_config: (ServiceConfig, Box<dyn ProxyService>) =
                (service_config, Box::new(aws_auth_service));
            Ok(aws_auth_config)
        }
        #[cfg(feature = "x509auth")]
        "x509" => {
            let ssl_auth_service =
                SslAuth::<Client<HttpsConnector<HttpConnector>>>::try_from(&service_config)?;
            let ssl_auth_config: (ServiceConfig, Box<dyn ProxyService>) =
                (service_config, Box::new(ssl_auth_service));
            Ok(ssl_auth_config)
        }
        #[cfg(feature = "hawkauth")]
        "hawk_auth" => {
            let hawk_auth_service =
                HawkAuth::<Client<HttpsConnector<HttpConnector>>>::try_from(&service_config)?;
            let hawk_auth_config: (ServiceConfig, Box<dyn ProxyService>) =
                (service_config, Box::new(hawk_auth_service));
            Ok(hawk_auth_config)
        }
        #[cfg(feature = "digestauth")]
        "digest_auth" => {
            let digest_auth_service =
                DigestAuth::<Client<HttpsConnector<HttpConnector>>>::try_from(&service_config)?;
            let digest_auth_config: (ServiceConfig, Box<dyn ProxyService>) =
                (service_config, Box::new(digest_auth_service));
            Ok(digest_auth_config)
        }
        "no_auth" => {
            let no_auth_service =
                NoAuth::<Client<HttpsConnector<HttpConnector>>>::try_from(&service_config)?;
            let no_auth_config: (ServiceConfig, Box<dyn ProxyService>) =
                (service_config, Box::new(no_auth_service));
            Ok(no_auth_config)
        }
        _ => Err(MarsError::ServiceNotRegistered),
    }
}
