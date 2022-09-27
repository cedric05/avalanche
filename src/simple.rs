use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::{convert::TryFrom, error::Error};

use crate::digestauth::DigestAuth;
#[cfg(feature = "hawkauth")]
use crate::hawkauth::HawkAuth;
use crate::noauth::NoAuth;
#[cfg(feature = "x509auth")]
use crate::x509::SslAuth;
#[cfg(feature = "awsauth")]
use crate::{awsauth::AwsAuth, error::MarsError};
use async_trait::async_trait;
use dashmap::{mapref::one::RefMut, DashMap};
use http::Response;
use hyper::{client::HttpConnector, Body, Client};
use hyper_tls::HttpsConnector;
use serde_json::Value;

use crate::{
    basicauth::BasicAuth,
    config::ServiceConfig,
    headerauth::HeaderAuth,
    project::{Project, ProjectHandler, ProxyService},
};

#[derive(Clone)]
struct SimpleProject {
    name: String,
    services: DashMap<String, (ServiceConfig, Box<dyn ProxyService>)>,
}

#[async_trait]
impl Project for SimpleProject {
    async fn is_project(&self, path: &str) -> bool {
        return self.name == path;
    }

    async fn get_service(
        &self,
        path: String,
    ) -> Option<RefMut<String, (ServiceConfig, Box<dyn ProxyService>)>> {
        self.services.get_mut(&path)
    }
}

#[derive(Clone)]
pub struct SimpleProjectHandler {
    projects: DashMap<String, Box<dyn Project>>,
}

// unsafe impl Send for SimpleProjectHandler {}

#[async_trait]
impl ProjectHandler for SimpleProjectHandler {
    async fn handle_request(
        &self,
        request: hyper::Request<Body>,
    ) -> Result<Response<Body>, Box<dyn Error>> {
        let uri = request.uri().clone();
        let uri = uri.path_and_query().unwrap().as_str();
        let mut url_split = uri.splitn(4, "/");
        let _host = url_split.next().ok_or(MarsError::UrlError)?;
        let project_key = url_split.next().ok_or(MarsError::UrlError)?;
        let service_key = url_split.next().ok_or(MarsError::UrlError)?;
        let (service, url_rest) = if service_key.contains('?') {
            let (service, url_rest) = service_key.split_once('?').unwrap();
            (service, "?".to_owned() + &url_rest)
        } else {
            let url_rest = url_split.next().unwrap_or("");
            (service_key, url_rest.to_owned())
        };
        // TODO, service has not extra backslash ('/'), service contains `?` also, which messes up everything
        if self.projects.contains_key(project_key) {
            let project = {
                let project = self
                    .projects
                    .get_mut(project_key)
                    .ok_or(MarsError::ServiceConfigError)?;
                project.clone()
            };
            let mut service_n_config = project
                .get_service(service.to_string())
                .await
                .ok_or(MarsError::ServiceConfigError)?;
            let (service_config, service) = service_n_config.value_mut();
            return service
                .handle_service(url_rest.as_str(), service_config, request)
                .await;
        }
        Ok(Response::builder().status(404).body(Body::empty()).unwrap())
    }
}

impl TryFrom<Value> for SimpleProject {
    type Error = MarsError;

    fn try_from(mut value: Value) -> Result<Self, Self::Error> {
        let value = value.as_object_mut().ok_or(MarsError::ServiceConfigError)?;

        let service_map = DashMap::new();
        for (service_key, service_config_value) in value.into_iter() {
            let service_config = service_config_value.take();
            let service_config: ServiceConfig = serde_json::from_value(service_config)
                .map_err(|_| MarsError::ServiceConfigError)?;
            match service_config.handler.handler_type.as_str() {
                #[cfg(feature = "basicauth")]
                "basic_auth" => {
                    let basic_auth_service =
                        BasicAuth::<Client<HttpsConnector<HttpConnector>>>::try_from(
                            &service_config,
                        )?;
                    let basic_auth_config: (ServiceConfig, Box<dyn ProxyService>) =
                        (service_config, Box::new(basic_auth_service));
                    service_map.insert(service_key.to_string(), basic_auth_config);
                }
                "header_auth" => {
                    let header_auth_service =
                        HeaderAuth::<Client<HttpsConnector<HttpConnector>>>::try_from(
                            &service_config,
                        )?;
                    let header_auth_config: (ServiceConfig, Box<dyn ProxyService>) =
                        (service_config, Box::new(header_auth_service));
                    service_map.insert(service_key.to_string(), header_auth_config);
                }
                #[cfg(feature = "awsauth")]
                "aws_auth" => {
                    let aws_auth_service =
                        AwsAuth::<Client<HttpsConnector<HttpConnector>>>::try_from(
                            &service_config,
                        )?;
                    let aws_auth_config: (ServiceConfig, Box<dyn ProxyService>) =
                        (service_config, Box::new(aws_auth_service));
                    service_map.insert(service_key.to_string(), aws_auth_config);
                }
                #[cfg(feature = "x509auth")]
                "x509" => {
                    let ssl_auth_service =
                        SslAuth::<Client<HttpsConnector<HttpConnector>>>::try_from(
                            &service_config,
                        )?;
                    let ssl_auth_config: (ServiceConfig, Box<dyn ProxyService>) =
                        (service_config, Box::new(ssl_auth_service));
                    service_map.insert(service_key.to_string(), ssl_auth_config);
                }
                #[cfg(feature = "hawkauth")]
                "hawk_auth" => {
                    let hawk_auth_service =
                        HawkAuth::<Client<HttpsConnector<HttpConnector>>>::try_from(
                            &service_config,
                        )?;
                    let hawk_auth_config: (ServiceConfig, Box<dyn ProxyService>) =
                        (service_config, Box::new(hawk_auth_service));
                    service_map.insert(service_key.to_string(), hawk_auth_config);
                }
                #[cfg(feature = "digestauth")]
                "digest_auth" => {
                    let digest_auth_service =
                        DigestAuth::<Client<HttpsConnector<HttpConnector>>>::try_from(
                            &service_config,
                        )?;
                    let digest_auth_config: (ServiceConfig, Box<dyn ProxyService>) =
                        (service_config, Box::new(digest_auth_service));
                    service_map.insert(service_key.to_string(), digest_auth_config);
                }
                "no_auth" => {
                    let no_auth_service =
                        NoAuth::<Client<HttpsConnector<HttpConnector>>>::try_from(&service_config)?;
                    let no_auth_config: (ServiceConfig, Box<dyn ProxyService>) =
                        (service_config, Box::new(no_auth_service));
                    service_map.insert(service_key.to_string(), no_auth_config);
                }
                _ => {
                    unimplemented!()
                }
            }
        }
        Ok(SimpleProject {
            name: "no meaning as of now".to_string(),
            services: service_map,
        })
    }
}
impl TryFrom<Value> for SimpleProjectHandler {
    type Error = MarsError;

    fn try_from(mut value: Value) -> Result<Self, Self::Error> {
        let value = value.as_object_mut().ok_or(MarsError::ServiceConfigError)?;
        let map = DashMap::new();
        for (project_key, project_config) in value {
            let project_config = project_config.take();
            let project = SimpleProject::try_from(project_config)?;
            let project: Box<dyn Project> = Box::new(project);
            map.insert(project_key.to_string(), project);
        }
        Ok(SimpleProjectHandler { projects: map })
    }
}

pub fn simple_project_handler(path: PathBuf) -> Result<SimpleProjectHandler, MarsError> {
    let mut file = fs::File::open(path).map_err(|_| MarsError::ServiceConfigError)?;
    let mut config = String::new();
    file.read_to_string(&mut config)
        .map_err(|_| MarsError::ServiceConfigError)?;
    let value: Value = json5::from_str(&config).map_err(|_| MarsError::ServiceConfigError)?;
    SimpleProjectHandler::try_from(value)
}
