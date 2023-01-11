mod auth;
use std::error::Error;

pub use auth::*;
#[cfg(feature = "config")]
mod common;
#[cfg(feature = "config")]
pub use common::*;
#[cfg(feature = "transform")]
mod transform;
use http::Response;
use hyper::Body;
#[cfg(feature = "transform")]
pub use transform::*;

pub fn response_from_status_message(
    status: u16,
    message: String,
) -> Result<Response<Body>, Box<dyn Error>> {
    Ok(Response::builder()
        .status(status)
        .body(Body::from(message))?)
}
