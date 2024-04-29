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
    let error = status >= 400;
    let response_body = serde_json::json!({
        "message": message,
        "error": error,
    })
    .to_string();
    Ok(Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .header("from-avalanche", "true")
        .body(Body::from(response_body))?)
}
