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

pub(crate) mod auth;
#[cfg(feature = "sql")]
pub(crate) mod db;

use std::{convert::Infallible, str::FromStr, sync::Arc};

use http::{header::HeaderName, HeaderValue, Request, Response};
use hyper::Body;
use project::ProjectManager;

#[cfg(feature = "sql")]
pub use db::get_db_project_manager;
pub use simple::get_json_project_manager;
use user::{AuthTokenStore, UserStore, UserTokenStore};

pub mod user;

pub async fn main_service(
    mut request: Request<Body>,
    project_handler: Arc<Box<dyn ProjectManager>>,
    user_store: Box<Arc<UserStore>>,
    user_token_store: Box<Arc<dyn UserTokenStore>>,
    auth_token_store: Box<Arc<dyn AuthTokenStore>>,
) -> Result<Response<Body>, Infallible> {
    // TODO, modifying header may not be accpetable for some
    // use uuid or some random generated
    let trace = "avalanceh-trace";
    request.headers_mut().insert(
        HeaderName::from_str("avalanche-trace").unwrap(),
        HeaderValue::from_str(trace).unwrap(),
    );
    let handle_request =
        project_handler.handle_request(request, user_store, user_token_store, auth_token_store);
    let response = match handle_request.await {
        Ok(result) => result,
        Err(error) => {
            log::error!("[{}] request ran into error= {:?}", trace, error);
            Response::builder().status(500).body(Body::empty()).unwrap()
        }
    };
    log::info!(
        "[{}] request completed with status {:?}",
        trace,
        response.status()
    );
    Ok(response)
}

pub async fn get_project_manager(args: &cli::Args) -> Arc<Box<dyn ProjectManager>> {
    #[cfg(feature = "sql")]
    if cfg!(feature = "sql") {
        return match &args.db {
            Some(db_url) => get_db_project_manager(&db_url)
                .await
                .expect("unable to connect to db"),
            None => {
                get_json_project_manager(args.config.clone().into()).expect("unable to load config")
            }
        };
    }
    get_json_project_manager(args.config.clone().into()).expect("unable to load config")
}
