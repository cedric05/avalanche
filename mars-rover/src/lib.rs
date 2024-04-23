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
mod cli;
#[cfg(feature = "sql")]
mod db;
mod project;
mod json_project_manager;
mod user;

use std::convert::Infallible;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use clap::Parser;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use mars_config::{AvalancheTrace, AVALANCHE_TRACE};
use user::{
    AuthToken,
    AuthTokenStore,
    InMemoryAuthTokenStore,
    SimpleUserTokenStore,
    //UserStore,
    UserTokenStore,
};

use http::{header::HeaderName, HeaderValue, Request, Response};
use hyper::Body;
use project::ProjectManager;
use std::net::Ipv4Addr;
use std::env;

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
    //    user_store: Box<Arc<UserStore>>,
    user_token_store: Box<Arc<dyn UserTokenStore>>,
    auth_token_store: Box<Arc<dyn AuthTokenStore>>,
) -> Result<Response<Body>, Infallible> {
    // TODO, modifying header may not be accpetable for some
    // use uuid or some random generated
    let trace = uuid::Uuid::new_v4().to_string();
    request.headers_mut().insert(
        HeaderName::from_str(AVALANCHE_TRACE).expect("impossible to fail"),
        HeaderValue::from_str(&trace).expect("impossible to fail"),
    );
    request.extensions_mut().insert(AvalancheTrace(trace.clone()));
    let handle_request = project_handler.handle_request(
        request,
        // user_store,
        user_token_store,
        auth_token_store,
    );


    let response = match handle_request.await {
        Ok(result) => result,
        Err(error) => {
            log::error!("[{}] request ran into error= {:?}", trace, error);
            Response::builder()
                .status(500)
                .body(Body::from(format!("error: {}", error)))
                .unwrap()
        }
    };
    log::info!(
        "[{}] request completed with status {:?}",
        trace,
        response.status()
    );
    Ok(response)
}


pub async fn start_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // TODO setup simple console output logger
    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.
    simple_logger::SimpleLogger::new()
        .with_colors(true)
        .with_level(log::LevelFilter::Info)
        .init()?;

    let args = cli::Args::parse();
    let project_handler = args.get_project_manager().await;

    // let user_store: Box<Arc<UserStore>> = Box::new(Arc::new(UserStore::default()));
    let user_token_store: Box<Arc<dyn UserTokenStore>> =
        Box::new(Arc::new(SimpleUserTokenStore::default()));
    let auth_token_store = InMemoryAuthTokenStore::default();
    auth_token_store.insert(AuthToken("hai".to_string()), "first".to_string());
    let auth_token_store: Box<Arc<dyn AuthTokenStore>> = Box::new(Arc::new(auth_token_store));

    let make_svc = make_service_fn(|_conn| {
        // This is the `Service` that will handle the connection.
        // `service_fn` is a helper to convert a function that
        // returns a Response into a `Service`.
        let project_handler = project_handler.clone();
        // let user_store = user_store.clone();
        let user_token_store = user_token_store.clone();
        let auth_token_store = auth_token_store.clone();

        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                //
                let project_handler = Arc::clone(&project_handler);
                // let user_store = user_store.clone();
                let user_token_store: Box<Arc<dyn UserTokenStore>> = user_token_store.clone();
                let auth_token_store: Box<Arc<dyn AuthTokenStore>> = auth_token_store.clone();
                async move {
                    //
                    hyper_service_fn(
                        req,
                        project_handler,
                        //user_store,
                        user_token_store,
                        auth_token_store,
                    )
                    .await
                }
            }))
        }
    });

    let port_key = "FUNCTIONS_CUSTOMHANDLER_PORT";
    let addr = match env::var(port_key) {
        Ok(val) => {
            let port = val.parse().expect("Custom Handler port is not a number!");
            SocketAddr::from((Ipv4Addr::LOCALHOST, port))
        },
        Err(_) => SocketAddr::from_str(&args.addr).expect("Unable to parse address from cli"),
    };

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}
