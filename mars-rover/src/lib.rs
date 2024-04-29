//! This is the main module of the Mars Rover application.
//!
//! It contains the entry point of the application (`main` function) and the `hyper_service_fn` function, which is an asynchronous function that processes incoming HTTP requests.
//!
//! The `hyper_service_fn` function takes in several parameters:
//! - `request`: The incoming HTTP request. It's mutable because the function modifies its headers.
//! - `project_handler`: An instance of a type that implements the `ProjectManager` trait. This object is responsible for handling the request.
//! - `user_token_store` and `auth_token_store`: Instances of types that implement the `UserTokenStore` and `AuthTokenStore` traits, respectively. They are likely used for storing and retrieving user and authentication tokens.
//!
//! Inside the `hyper_service_fn` function, a new UUID is generated and added as a header to the request. This UUID is likely used as a trace ID for logging and debugging purposes. The UUID is also added to the request's extensions, which is a way to attach additional data to the request.
//!
//! The `handle_request` method of the `project_handler` is then called with the request and the token stores as parameters. This method is awaited because it's asynchronous, which means it might perform some IO operations, such as sending a network request or querying a database.
//!
//! The `hyper_service_fn` function returns a `Result` containing a `Response` if the request is successfully processed, or an `Infallible` error if the request cannot be processed.
//!
//! The `main` function is the entry point of the application. It sets up the logging, parses command line arguments, initializes the project handler, token stores, and the HTTP server. It then starts the server and listens for incoming HTTP requests.
//!
//! # Examples
//!
//!
#[cfg(feature = "sql")]
pub mod db;
pub mod file;
pub mod project;

use std::convert::Infallible;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use auth::response_from_status_message;
use http::{header::HeaderName, HeaderValue, Request, Response};
use hyper::service::{make_service_fn, service_fn};
use hyper::Body;
use hyper::Server;
use mars_config::{AvalancheTrace, AVALANCHE_TRACE};
use project::ProjectManager;

pub use mars_request_transform as auth;

/// `hyper_service_fn` is an asynchronous function that processes incoming HTTP requests.
///
/// It takes in several parameters:
/// - `request`: The incoming HTTP request. It's mutable because the function modifies its headers.
/// - `project_handler`: An instance of a type that implements the `ProjectManager` trait. This object is responsible for handling the request.
/// - `user_token_store` and `auth_token_store`: Instances of types that implement the `UserTokenStore` and `AuthTokenStore` traits, respectively. They are likely used for storing and retrieving user and authentication tokens.
///
/// Inside the function, a new UUID is generated and added as a header to the request. This UUID is likely used as a trace ID for logging and debugging purposes. The UUID is also added to the request's extensions, which is a way to attach additional data to the request.
///
/// The `handle_request` method of the `project_handler` is then called with the request and the token stores as parameters. This method is awaited because it's asynchronous, which means it might perform some IO operations, such as sending a network request or querying a database.
///
/// # Examples
///
/// ```rust
/// let response = hyper_service_fn(request, project_handler, user_token_store, auth_token_store).await;
/// ```
///
/// # Errors
///
/// This function will return an error if the request cannot be processed, for example due to invalid tokens or network issues.

async fn hyper_service_fn(
    mut request: Request<Body>,
    project_handler: Arc<Box<dyn ProjectManager>>,
) -> Result<Response<Body>, Infallible> {
    // using uuid as trace
    let trace = uuid::Uuid::new_v4().to_string();
    // TODO, modifying header may not be accpetable for some
    request.headers_mut().insert(
        HeaderName::from_str(AVALANCHE_TRACE).expect("impossible to fail"),
        HeaderValue::from_str(&trace).expect("impossible to fail"),
    );
    request
        .extensions_mut()
        .insert(AvalancheTrace(trace.clone()));

    match project_handler.handle_request(request).await {
        Ok(result) => {
            log::info!(
                "[{}] request completed with status {:?}",
                trace,
                result.status()
            );
            Ok(result)
        }
        Err(error) => {
            log::error!("[{}] request ran into error= {:?}", trace, error);
            Ok(response_from_status_message(500, format!("error: {}", error)).unwrap())
        }
    }
}

pub async fn start_server(addr: SocketAddr, project_handler:  Arc<Box<dyn ProjectManager>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {


    let make_svc = make_service_fn(|_conn| {
        // This is the `Service` that will handle the connection.
        // `service_fn` is a helper to convert a function that
        // returns a Response into a `Service`.
        let project_handler = project_handler.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                //
                let project_handler = Arc::clone(&project_handler);
                async move {
                    //
                    hyper_service_fn(req, project_handler).await
                }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}
