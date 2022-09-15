pub(crate) mod awsauth;
pub(crate) mod basicauth;
pub(crate) mod config;
pub(crate) mod error;
pub(crate) mod headerauth;
pub(crate) mod project;
pub(crate) mod simple;
pub(crate) mod x509;

#[macro_use]
pub(crate) mod utils;

use std::{convert::Infallible, sync::Arc};

use http::{Request, Response};
use hyper::Body;
use project::ProjectHandler;
use simple::SimpleProjectHandler;

pub use simple::simple_project_handler;

pub async fn main_service(
    request: Request<Body>,
    project_handler: Arc<SimpleProjectHandler>,
) -> Result<Response<Body>, Infallible> {
    let handle_request = project_handler.handle_request(request);
    match handle_request.await {
        Ok(result) => Ok(result),
        Err(error) => {
            println!("error is {:?}", error);
            Ok(Response::builder().status(500).body(Body::empty()).unwrap())
        }
    }
}
