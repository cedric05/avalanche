use std::{error::Error, future::Future, pin::Pin, sync::Arc};

use async_trait::async_trait;
use dashmap::mapref::one::RefMut;
use dyn_clone::{clone_trait_object, DynClone};
use http::{Request, Response};
use hyper::{service::Service, Body};

use crate::{
    config::ServiceConfig,
    user::{AuthTokenStore, UserStore, UserTokenStore},
};

/// project
/// project has two main variables
/// 1. project identifier
/// 2. service identifier
///
///
///

#[async_trait]
pub trait Project: Sync + Send + DynClone {
    async fn is_project(&self, path: &str) -> bool;

    async fn get_service(
        &self,
        path: String,
    ) -> Option<RefMut<String, (ServiceConfig, Box<dyn ProxyService>)>>;
}

clone_trait_object!(Project);

#[async_trait]
pub trait ProjectHandler {
    async fn handle_request(
        &self,
        request: hyper::Request<Body>,
        user_store: Box<Arc<UserStore>>,
        user_token_store: Box<Arc<dyn UserTokenStore>>,
        auth_token_store: Box<Arc<dyn AuthTokenStore>>,
    ) -> Result<Response<Body>, Box<dyn Error>>;
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
