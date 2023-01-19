use std::{error::Error, sync::Arc};

use async_trait::async_trait;
use clap::Result;
use dashmap::mapref::one::RefMut;
use dyn_clone::{clone_trait_object, DynClone};
use http::Response;
use hyper::Body;
use mars_config::{MarsError, AVALANCHE_TOKEN};

use crate::user::{
    AuthToken,
    AuthTokenStore,
    //UserStore,
    UserTokenStore,
};
use hyper::service::Service;
use mars_request_transform::{response_from_status_message, ProxyService, ProxyUrlPath};

/// project
/// project has two main variables
/// 1. project identifier
/// 2. service identifier
///
///
///

#[async_trait]
pub(crate) trait ProjectHandler: Sync + Send + DynClone {
    async fn is_project(&self, path: &str) -> bool;

    async fn get_service(
        &self,
        path: String,
    ) -> Result<Option<RefMut<String, ProxyService>>, Box<dyn Error>>;

    async fn auth_configured(&self) -> bool;
}

clone_trait_object!(ProjectHandler);

#[async_trait]
pub(crate) trait ProjectManager: Sync + Send {
    async fn handle_request(
        &self,
        mut request: hyper::Request<Body>,
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
                let mut service_pair =
                    project
                        .get_service(service.to_string())
                        .await?
                        .ok_or_else(|| {
                            MarsError::ServiceConfigError(format!(
                                "project `{}`'s subproject `{}` configured incorrectly or not",
                                project_key, service_key
                            ))
                        })?;
                request.extensions_mut().insert(ProxyUrlPath(url_rest));
                let service = service_pair.value_mut();
                // TODO Handle errors or waits
                futures::future::poll_fn(|cx| service.poll_ready(cx))
                    .await
                    .unwrap_or(());
                match service.call(request).await {
                    Err(resp) => response_from_status_message(
                        500,
                        format!("request to proxy ran into error: `{}`", resp),
                    ),
                    Ok(resp) => Ok(resp),
                }
            }
            None => return response_from_status_message(404, "project not found".into()),
        }
    }
    async fn get_project(
        &self,
        project_key: String,
    ) -> Result<Option<Arc<Box<dyn ProjectHandler>>>, Box<dyn Error>>;
}
