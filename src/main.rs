use std::convert::Infallible;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use clap::Parser;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use mars_rover::user::{
    AuthToken,
    AuthTokenStore,
    SimpleAuthTokenStore,
    SimpleUserTokenStore,
    //UserStore,
    UserTokenStore,
};
use mars_rover::{get_project_manager, main_service};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // TODO setup simple console output logger
    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.
    simple_logger::SimpleLogger::new()
        .with_colors(true)
        .with_level(log::LevelFilter::Info)
        .init()?;

    let args = mars_rover::cli::Args::parse();
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
