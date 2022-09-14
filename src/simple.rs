use std::{convert::TryFrom, error::Error};

use crate::x509::SslAuth;
use crate::{awsauth::AwsAuth, error::MarsError};
use async_trait::async_trait;
use dashmap::{mapref::one::RefMut, DashMap};
use http::Response;
use hyper::{client::HttpConnector, Body, Client};
use hyper_tls::HttpsConnector;
use serde_json::{json, Value};

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
        &mut self,
        request: hyper::Request<Body>,
    ) -> Result<Response<Body>, Box<dyn Error>> {
        let uri = request.uri().clone();
        let uri = uri.path_and_query().unwrap().as_str();
        let mut url_split = uri.splitn(4, "/");
        let _host = url_split.next().ok_or(MarsError::UrlError)?;
        let project = url_split.next().ok_or(MarsError::UrlError)?;
        let service = url_split.next().ok_or(MarsError::UrlError)?;
        let rest = url_split.next().unwrap_or("");
        if self.projects.contains_key(project) {
            let get_mut = self.projects.get_mut(project);
            let project = get_mut.ok_or(MarsError::ServiceConfigError)?;
            let mut service_n_config = project
                .get_service(service.to_string())
                .await
                .ok_or(MarsError::ServiceConfigError)?;
            let (service_config, service) = service_n_config.value_mut();
            return service.handle_service(rest, service_config, request).await;
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
                "aws_auth" => {
                    let aws_auth_service =
                        AwsAuth::<Client<HttpsConnector<HttpConnector>>>::try_from(
                            &service_config,
                        )?;
                    let aws_auth_config: (ServiceConfig, Box<dyn ProxyService>) =
                        (service_config, Box::new(aws_auth_service));
                    service_map.insert(service_key.to_string(), aws_auth_config);
                }
                "x509" => {
                    let ssl_auth_service =
                        SslAuth::<Client<HttpsConnector<HttpConnector>>>::try_from(
                            &service_config,
                        )?;
                    let aws_auth_config: (ServiceConfig, Box<dyn ProxyService>) =
                        (service_config, Box::new(ssl_auth_service));
                    service_map.insert(service_key.to_string(), aws_auth_config);
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

pub fn simple_project_handler() -> Result<SimpleProjectHandler, MarsError> {
    let value = json!(
    {
        "first": {
            "sample1":{
                "url":"http://httpbin.org/",
                "method":"ANY",
                "query_params":[{"key":"test",
                    "value":"test",
                    "action":"Add"
                }],
                "headers":[{
                    "key":"test",
                    "value":"test",
                    "action":"Add"}
                ],
                "handler":{
                    "params":{
                        "password":"password",
                        "username":"prasanth"
                    },
                    "handler_type":"basic_auth"

                }
            },
            "sample2":{
                "url":"http://httpbin.org/get",
                "method":"ANY",
                "query_params":[{
                    "key":"test",
                    "value":"test",
                    "action":"Add"
                }],
                "headers":[{
                    "key":"test",
                    "value":
                    "test",
                    "action":"Add"
                }],
                "handler":{
                    "params":{"key":"rama","value":"ranga"},
                    "handler_type":"header_auth"
                }
            },
            "aws_auth":{
                "url":"https://ec2.amazonaws.com/",
                "method":"ANY",
                "query_params":[],
                "headers":[],
                "handler":{
                    "params":{
                        "access_key": "",
                        "secret_key":"",
                        "region":"us-east-1",
                        "service":"ec2"
                    },
                    "handler_type":"aws_auth"
                }
            },
            "ssl":{
                "url":"https://client.badssl.com/",
                "method":"ANY",
                "query_params":[],
                "headers":[],
                "handler":{
                    "params":{
                        "pkcs12": "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",
                        "pkcs12_password": "badssl.com"
                    },
                    "handler_type":"x509"
                }
            }
        }
    });
    SimpleProjectHandler::try_from(value)
}
