use std::{error::Error, future::Future, pin::Pin, sync::Arc};

use async_trait::async_trait;
use clap::Result;
use dashmap::mapref::one::RefMut;
use dyn_clone::{clone_trait_object, DynClone};
use http::{Request, Response};
use hyper::{service::Service, Body};

use crate::config::ServiceConfig;
use crate::error::MarsError;
use crate::user::{
    AuthToken,
    AuthTokenStore,
    //UserStore,
    UserTokenStore,
};

/// project
/// project has two main variables
/// 1. project identifier
/// 2. service identifier
///
///
///

pub (crate) const AVALANCHE_TOKEN: &str = "avalanche-token";

#[async_trait]
pub (crate) trait ProjectHandler: Sync + Send + DynClone {
    async fn is_project(&self, path: &str) -> bool;

    async fn get_service(
        &self,
        path: String,
    ) -> Result<Option<RefMut<String, (ServiceConfig, Box<dyn ProxyService>)>>, Box<dyn Error>>;

    async fn auth_configured(&self) -> bool;
}

clone_trait_object!(ProjectHandler);

fn response_from_status_message(
    status: u16,
    message: String,
) -> Result<Response<Body>, Box<dyn Error>> {
    Ok(Response::builder()
        .status(status)
        .body(Body::from(message))?)
}

#[async_trait]
pub (crate) trait ProjectManager: Sync + Send {
    async fn handle_request(
        &self,
        request: hyper::Request<Body>,
        user_token_store: Box<Arc<dyn UserTokenStore>>,
        auth_token_store: Box<Arc<dyn AuthTokenStore>>,
    ) -> Result<Response<Body>, Box<dyn Error>> {
        log::info!("recieved a request {:?}", request.uri());
        let uri = request.uri().clone();
        let uri = uri.path_and_query().unwrap().as_str();
        let mut url_split = uri.splitn(4, '/');
        let _host = url_split.next().ok_or_else(||MarsError::UrlError(
            "marsrover url should contain https://<host>/<project>/<subproject>/<rest>. host it is missing".to_string()
        ))?;
        let project_key = url_split.next().ok_or_else(||MarsError::UrlError(
            "marsrover url should contain https://<host>/<project>/<subproject>/<rest>. project it is missing".to_string()
        ))?;

        let project = self.get_project(project_key.to_string()).await?;
        match project {
            Some(project) => {
                if project.auth_configured().await {
                    let avalanche_token = if let Some(avalanche_token) =
                        request.headers().get(AVALANCHE_TOKEN).cloned()
                    {
                        avalanche_token
                    } else {
                        return response_from_status_message(
                            401,
                            "avalanche token not provided".into(),
                        );
                    };
                    let avalanche_token =
                        AuthToken(String::from_utf8(avalanche_token.as_bytes().to_vec())?);
                    if !(user_token_store.exists(&avalanche_token)
                        || auth_token_store.exists(&avalanche_token, project_key))
                    {
                        return response_from_status_message(
                            401,
                            "avalanche token not valid".into(),
                        );
                    }
                }

                // TODO, service has not extra backslash ('/'), service contains `?` also, which messes up everything
                let service_key = url_split.next().ok_or_else(||MarsError::UrlError(
                    "marsrover url should contain https://<host>/<project>/<subproject>/<rest>. subproject is missing".to_string()
                ))?;
                // TODO
                // inplace of service_key contains, we may have to go with startswith
                let (service, url_rest) = if service_key.contains('?') {
                    let (service, url_rest) = service_key.split_once('?').unwrap();
                    (service, "?".to_owned() + url_rest)
                } else {
                    let url_rest = url_split.next().unwrap_or("");
                    (service_key, url_rest.to_owned())
                };
                let mut service_n_config = project
                    .get_service(service.to_string())
                    .await?
                    .ok_or_else(|| {
                        MarsError::ServiceConfigError(format!(
                            "project `{}`'s subproject `{}` configured incorrectly or not",
                            project_key, service_key
                        ))
                    })?;
                let (service_config, service) = service_n_config.value_mut();
                return service
                    .handle_service(url_rest.as_str(), service_config, request)
                    .await;
            }
            None => return response_from_status_message(404, "project not found".into()),
        }
    }
    async fn get_project(
        &self,
        project_key: String,
    ) -> Result<Option<Arc<Box<dyn ProjectHandler>>>, Box<dyn Error>>;
}

#[async_trait]
pub (crate) trait ProxyService:
    Sync
    + Send
    + DynClone
    + Service<
        Request<Body>,
        Response = Response<Body>,
        Error = hyper::Error,
        Future = Pin<Box<dyn Future<Output = Result<Response<Body>, hyper::Error>> + Send>>,
    >
{
    async fn handle_service(
        &mut self,
        url: &str,
        service_config: &ServiceConfig,
        request: hyper::Request<Body>,
    ) -> Result<Response<Body>, Box<dyn Error>>;
}

clone_trait_object!(ProxyService);
