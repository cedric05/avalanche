#[cfg(feature = "awsauth")]
pub(crate) mod awsauth;
#[cfg(feature = "basicauth")]
pub(crate) mod basicauth;
pub mod cli;
pub(crate) mod config;
#[cfg(feature = "digestauth")]
pub(crate) mod digestauth;
pub(crate) mod error;
#[cfg(feature = "hawkauth")]
pub(crate) mod hawkauth;
pub(crate) mod headerauth;
pub(crate) mod noauth;
pub(crate) mod project;
pub(crate) mod simple;
#[cfg(feature = "x509auth")]
pub(crate) mod x509;
#[macro_use]
pub(crate) mod utils;

use std::{convert::Infallible, sync::Arc};

use http::{Request, Response};
use hyper::Body;
use project::ProjectHandler;
use simple::SimpleProjectHandler;

pub use simple::simple_project_handler;
use user::{AuthTokenStoreT, UserStore, UserTokenStoreT};

pub mod user;

pub async fn main_service(
    request: Request<Body>,
    project_handler: Arc<SimpleProjectHandler>,
    user_store: Box<Arc<UserStore>>,
    user_token_store: Box<Arc<dyn UserTokenStoreT>>,
    auth_token_store: Box<Arc<dyn AuthTokenStoreT>>,
) -> Result<Response<Body>, Infallible> {
    let handle_request =
        project_handler.handle_request(request, user_store, user_token_store, auth_token_store);
    match handle_request.await {
        Ok(result) => Ok(result),
        Err(error) => {
            println!("error is {:?}", error);
            Ok(Response::builder().status(500).body(Body::empty()).unwrap())
        }
    }
}
