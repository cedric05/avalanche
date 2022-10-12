use std::{error::Error, future::Future, pin::Pin, sync::Arc};

use async_trait::async_trait;
use clap::Result;
use dashmap::mapref::one::RefMut;
use dyn_clone::{clone_trait_object, DynClone};
use http::{Request, Response};
use hyper::{service::Service, Body};

use crate::config::ServiceConfig;
use crate::error::MarsError;
use crate::user::{AuthToken, AuthTokenStore, UserStore, UserTokenStore};

/// project
/// project has two main variables
/// 1. project identifier
/// 2. service identifier
///
///
///

#[async_trait]
pub trait ProjectHandler: Sync + Send + DynClone {
    async fn is_project(&self, path: &str) -> bool;

    async fn get_service(
        &self,
        path: String,
    ) -> Result<Option<RefMut<String, (ServiceConfig, Box<dyn ProxyService>)>>, Box<dyn Error>>;

    async fn auth_configured(&self) -> bool;
}

clone_trait_object!(ProjectHandler);

#[async_trait]
pub trait ProjectManager {
    async fn handle_request(
        &self,
        request: hyper::Request<Body>,
        _user_store: Box<Arc<UserStore>>,
        user_token_store: Box<Arc<dyn UserTokenStore>>,
        auth_token_store: Box<Arc<dyn AuthTokenStore>>,
    ) -> Result<Response<Body>, Box<dyn Error>> {
        log::info!("recieved a request {:?}", request.uri());
        let uri = request.uri().clone();
        let uri = uri.path_and_query().unwrap().as_str();
        let mut url_split = uri.splitn(4, "/");
        let _host = url_split.next().ok_or(MarsError::UrlError)?;
        let project_key = url_split.next().ok_or(MarsError::UrlError)?;

        let project = self.get_project(project_key.to_string()).await?;
        match project {
            Some(project) => {
                if project.auth_configured().await {
                    let avalanche_token = if let Some(avalanche_token) =
                        request.headers().get("avalanche-token").cloned()
                    {
                        avalanche_token
                    } else {
                        return Ok(Response::builder().status(401).body(Body::empty()).unwrap());
                    };
                    let avalanche_token =
                        AuthToken(String::from_utf8(avalanche_token.as_bytes().to_vec())?);
                    if !(user_token_store.exists(&avalanche_token)
                        || auth_token_store.exists(&avalanche_token, project_key))
                    {
                        return Ok(Response::builder().status(401).body(Body::empty()).unwrap());
                    }
                }

                // TODO, service has not extra backslash ('/'), service contains `?` also, which messes up everything
                let service_key = url_split.next().ok_or(MarsError::UrlError)?;
                let (service, url_rest) = if service_key.contains('?') {
                    let (service, url_rest) = service_key.split_once('?').unwrap();
                    (service, "?".to_owned() + &url_rest)
                } else {
                    let url_rest = url_split.next().unwrap_or("");
                    (service_key, url_rest.to_owned())
                };
                let mut service_n_config = project
                    .get_service(service.to_string())
                    .await?
                    .ok_or(MarsError::ServiceConfigError)?;
                let (service_config, service) = service_n_config.value_mut();
                return service
                    .handle_service(url_rest.as_str(), service_config, request)
                    .await;
            }
            None => Ok(Response::builder().status(404).body(Body::empty()).unwrap()),
        }
    }
    async fn get_project(
        &self,
        project_key: String,
    ) -> Result<Option<Box<dyn ProjectHandler>>, Box<dyn Error>>;
}

#[async_trait]
pub trait ProxyService:
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
