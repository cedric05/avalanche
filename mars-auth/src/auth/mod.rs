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
