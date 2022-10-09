use std::convert::Infallible;
use std::sync::Arc;

use clap::Parser;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use mars_rover::user::{
    AuthToken, AuthTokenStoreT, AuthTokensStore, UserStore, UserTokenStore, UserTokenStoreT,
};
use mars_rover::{main_service, simple_project_handler};

fn conncurrent<T: Default>() -> Box<Arc<T>> {
    let t = T::default();
    Box::new(Arc::new(t))
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.
    let args = mars_rover::cli::Args::parse();
    let project_handler = Arc::new(simple_project_handler(args.config.into())?);
    let user_store: Box<Arc<UserStore>> = conncurrent();
    let user_token_store: Box<Arc<dyn UserTokenStoreT>> =
        Box::new(Arc::new(UserTokenStore::default()));
    let auth_token_store = AuthTokensStore::default();
    auth_token_store
        .project_token
        .insert(AuthToken("hai".to_string()), "first".to_string());
    let auth_token_store: Box<Arc<dyn AuthTokenStoreT>> = Box::new(Arc::new(auth_token_store));

    let make_svc = make_service_fn(|_conn| {
        // This is the `Service` that will handle the connection.
        // `service_fn` is a helper to convert a function that
        // returns a Response into a `Service`.
        let project_handler = project_handler.clone();
        let user_store = user_store.clone();
        let user_token_store = user_token_store.clone();
        let auth_token_store = auth_token_store.clone();

        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                //
                let project_handler = Arc::clone(&project_handler);
                let user_store = user_store.clone();
                let user_token_store: Box<Arc<dyn UserTokenStoreT>> = user_token_store.clone();
                let auth_token_store: Box<Arc<dyn AuthTokenStoreT>> = auth_token_store.clone();
                async move {
                    //
                    main_service(
                        req,
                        project_handler,
                        user_store,
                        user_token_store,
                        auth_token_store,
                    )
                    .await
                }
            }))
        }
    });

    let addr = ([127, 0, 0, 1], args.port).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}
