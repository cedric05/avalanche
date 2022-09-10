use std::convert::Infallible;
use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use project::{simple_project_handler, ProjectHandler, SimpleProjectHandler};

mod basicauth;
mod config;
mod project;

async fn main_service(
    request: Request<Body>,
    project_handler: Arc<tokio::sync::Mutex<SimpleProjectHandler>>,
) -> Result<Response<Body>, Infallible> {
    let mut simple_project_handler = project_handler.try_lock().unwrap();
    let handle_request = simple_project_handler.handle_request(request);
    match handle_request.await {
        Ok(result) => Ok(result),
        Err(error) => {
            println!("error is {:?}", error);
            Ok(Response::builder().status(500).body(Body::empty()).unwrap())
        }
    }
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.
    let project_handler = simple_project_handler();
    let project_handler = Arc::new(tokio::sync::Mutex::new(project_handler));
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
                    main_service(req, project_handler).await
                }
            }))
        }
    });

    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}
