mod cli;
#[cfg(feature = "sql")]
mod db;
mod project;
mod simple;
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
    SimpleAuthTokenStore,
    SimpleUserTokenStore,
    //UserStore,
    UserTokenStore,
};

use http::{header::HeaderName, HeaderValue, Request, Response};
use hyper::Body;
use project::ProjectManager;

pub use mars_request_transform as auth;

async fn main_service(
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

async fn get_project_manager(args: &cli::Args) -> Arc<Box<dyn ProjectManager>> {
    #[cfg(feature = "sql")]
    if cfg!(feature = "sql") {
        return match &args.db {
            Some(db_url) => db::get_db_project_manager(db_url)
                .await
                .expect("unable to connect to db"),
            None => simple::get_json_project_manager(args.config.clone().into())
                .expect("unable to load config"),
        };
    }
    simple::get_json_project_manager(args.config.clone().into()).expect("unable to load config")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // TODO setup simple console output logger
    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.
    simple_logger::SimpleLogger::new()
        .with_colors(true)
        .with_level(log::LevelFilter::Info)
        .init()?;

    let args = cli::Args::parse();
    let project_handler = get_project_manager(&args).await;

    // let user_store: Box<Arc<UserStore>> = Box::new(Arc::new(UserStore::default()));
    let user_token_store: Box<Arc<dyn UserTokenStore>> =
        Box::new(Arc::new(SimpleUserTokenStore::default()));
    let auth_token_store = SimpleAuthTokenStore::default();
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
                    main_service(
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

    let addr = SocketAddr::from_str(&args.addr)?;

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}
