/// This module contains the authentication implementations for different authentication methods.
/// 
/// The available authentication methods are:
/// - AWS authentication (`awsauth`)
/// - Basic authentication (`basicauth`)
/// - Digest authentication (`digestauth`)
/// - Hawk authentication (`hawkauth`)
/// - Header authentication (`headerauth`)
/// - X509 authentication (`x509`)
/// 
/// Additionally, this module also includes the `service` module, which provides configuration related functionality.
/// 
/// To use any of the authentication methods, enable the corresponding feature flag in your Cargo.toml file.
/// 
/// Example:
/// ```toml
/// [dependencies]
/// mars-request-transform = { version = "1.0", features = ["awsauth", "basicauth"] }
/// ```
#[cfg(feature = "awsauth")]
pub mod awsauth;

#[cfg(feature = "basicauth")]
pub mod basicauth;

#[cfg(feature = "digestauth")]
pub mod digestauth;

#[cfg(feature = "hawkauth")]
pub mod hawkauth;

pub mod headerauth;

#[cfg(feature = "x509auth")]
pub mod x509;

#[cfg(feature = "config")]
pub mod service;

#[cfg(feature = "config")]
pub use service::*;
