use std::{collections::HashMap, str::FromStr, vec};

use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};
use http::{header::HeaderName, HeaderValue, Request, Response, Uri};
use hyper::{client::HttpConnector, Body};
use serde_json::json;
use url::Url;

use crate::config::{Action, Header, ProxyParams, ServiceConfig, UrlParam};

lazy_static::lazy_static! {
    static ref HOP_HEADERS: Vec<HeaderName> = vec![
        HeaderName::from_str("Connection").unwrap(),
        HeaderName::from_str("Keep-Alive").unwrap(),
        HeaderName::from_str("Proxy-Authenticate").unwrap(),
        HeaderName::from_str("Proxy-Authorization").unwrap(),
        HeaderName::from_str("Te").unwrap(),
        HeaderName::from_str("Trailers").unwrap(),
        HeaderName::from_str("Transfer-Encoding").unwrap(),
        HeaderName::from_str("Upgrade").unwrap(),
    ];
}
/// project
/// project has two main variables
/// 1. project identifier
/// 2. service identifier
///
///
///

#[async_trait]
trait Project: Sync + Send + DynClone {
    async fn is_project(&self, path: &str) -> bool;

    async fn is_service(&self, path: &str) -> Option<&(ServiceConfig, Box<dyn ProxyService>)>;
}

clone_trait_object!(Project);

#[async_trait]
pub trait ProjectHandler {
    async fn handle_request(&self, request: hyper::Request<Body>) -> Response<Body>;
}

#[async_trait]
trait ProxyService: Sync + Send + DynClone {
    async fn handle_service(
        &self,
        url: &str,
        service_config: &ServiceConfig,
        request: hyper::Request<Body>,
    ) -> Response<Body>;
}

clone_trait_object!(ProxyService);

#[derive(Clone)]
struct BasicAuth {
    client: hyper::Client<HttpConnector>,
}

impl ServiceConfig {
    fn get_updated_request(&self, rest: &str, req: &mut Request<Body>) {
        let uri = Url::from_str(&self.url.clone()).unwrap();
        let uri = uri.join(rest).unwrap();
        let params: Vec<(String, String)> = self
            .query_params
            .iter()
            .filter(|x| x.action == Action::Add)
            .map(|x| (x.key.clone(), x.value.clone()))
            .collect();
        let uri: Url = Url::parse_with_params(&uri.to_string(), &params).unwrap();
        *req.uri_mut() = Uri::from_str(&uri.to_string()).unwrap();
        let headers_mut = req.headers_mut();
        for header in &self.headers {
            if header.action == Action::Add {
                let key = HeaderName::from_str(&header.key).unwrap();
                headers_mut.append(key, HeaderValue::from_str(&header.value).unwrap());
            }
        }
        for header in HOP_HEADERS.iter() {
            headers_mut.remove(header);
        }
    }
}

#[async_trait]
impl ProxyService for BasicAuth {
    async fn handle_service(
        &self,
        url: &str,
        service_config: &ServiceConfig,
        request: hyper::Request<Body>,
    ) -> Response<Body> {
        let mut request = request;
        service_config.get_updated_request(url, &mut request);
        let headers_mut = request.headers_mut();
        let username = service_config
            .handler
            .params
            .get("username")
            .unwrap()
            .as_str()
            .unwrap();
        let password = service_config
            .handler
            .params
            .get("password")
            .unwrap()
            .as_str()
            .unwrap();
        headers_mut.append(
            "Authentication",
            HeaderValue::from_str(&base64::encode(format!("{}:{}", username, password))).unwrap(),
        );
        headers_mut.remove("host");
        println!("request is {:?}", request);
        let response = self.client.request(request).await.unwrap();
        response
    }
}

#[derive(Clone)]
struct SimpleProject<'a> {
    name: &'a str,
    services: HashMap<&'a str, (ServiceConfig, Box<dyn ProxyService>)>,
}

#[async_trait]
impl<'a> Project for SimpleProject<'a> {
    async fn is_project(&self, path: &str) -> bool {
        return self.name == path;
    }

    async fn is_service(&self, path: &str) -> Option<&(ServiceConfig, Box<dyn ProxyService>)> {
        self.services.get(path)
    }
}

#[derive(Clone)]
pub struct SimpleProjectHandler {
    projects: Vec<Box<dyn Project>>,
}

#[async_trait]
impl ProjectHandler for SimpleProjectHandler {
    async fn handle_request(&self, request: hyper::Request<Body>) -> Response<Body> {
        let uri = request.uri().clone();
        let uri = uri.path_and_query().unwrap().as_str();
        let mut url_split = uri.splitn(4, "/");
        let _host = url_split.next().unwrap();
        let project = url_split.next().unwrap();
        let service = url_split.next().unwrap();
        let rest = url_split.next().unwrap();
        println!("project is {} and service is {}", project, service);
        for a_project in &self.projects {
            if a_project.is_project(project).await {
                let service_config = a_project.is_service(service).await;
                if service_config.is_some() {
                    let (service_config, proxy_service) = service_config.as_ref().unwrap();
                    return proxy_service
                        .handle_service(rest, service_config, request)
                        .await;
                }
            }
        }
        Response::builder().status(404).body(Body::empty()).unwrap()
    }
}

pub fn simple_project_handler() -> SimpleProjectHandler {
    let basic_auth: Box<dyn ProxyService> = Box::new(BasicAuth {
        client: hyper::Client::new(),
    });
    let haha = (
        ServiceConfig {
            method: crate::config::Method::ANY,
            query_params: vec![UrlParam {
                key: "test".to_string(),
                value: "test".to_string(),
                action: Action::Add,
            }],
            headers: vec![Header {
                key: "test".to_string(),
                value: "test".to_string(),
                action: Action::Add,
            }],
            url: "http://httpbin.org/get".to_string(),
            handler: ProxyParams {
                params: json! ({
                    "username":"prasanth",
                    "password": "password"
                }),
                handler_type: "basic_auth".to_string(),
            },
        },
        basic_auth,
    );
    SimpleProjectHandler {
        projects: vec![Box::new(SimpleProject {
            name: "first",
            services: HashMap::from_iter([("sample1", haha)]),
        })],
    }
}
