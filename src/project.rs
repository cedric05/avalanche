use std::{error::Error, fmt::Display, future::Future, pin::Pin, str::FromStr, vec};

use crate::basicauth::{BasicAuth, BasicAuthLayer};
use async_trait::async_trait;
use dashmap::{mapref::one::RefMut, DashMap};
use dyn_clone::{clone_trait_object, DynClone};
use http::{header::HeaderName, HeaderValue, Request, Response, Uri};
use hyper::{client::HttpConnector, service::Service, Body, Client};
use hyper_tls::HttpsConnector;
use serde_json::json;
use tower::ServiceBuilder;
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

#[derive(Debug)]
pub enum MarsError {
    UrlError,
    ServiceConfigError,
}

impl Display for MarsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarsError::UrlError => f.write_str("urlerror"),
            MarsError::ServiceConfigError => f.write_str("service config error"),
        }
    }
}

impl std::error::Error for MarsError {}

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

    async fn get_service(
        &self,
        path: String,
    ) -> Option<RefMut<'async_trait, String, (ServiceConfig, Box<dyn ProxyService>)>>;
}

clone_trait_object!(Project);

#[async_trait]
pub trait ProjectHandler {
    async fn handle_request(
        &mut self,
        request: hyper::Request<Body>,
    ) -> Result<Response<Body>, Box<dyn Error>>;
}

#[async_trait]
trait ProxyService:
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

#[async_trait]
impl ProxyService for BasicAuth<hyper::Client<HttpsConnector<HttpConnector>>> {
    async fn handle_service(
        &mut self,
        url: &str,
        service_config: &ServiceConfig,
        request: hyper::Request<Body>,
    ) -> Result<Response<Body>, Box<dyn Error>> {
        let mut request = request;
        service_config.get_updated_request(url, &mut request)?;
        let response = self.call(request).await?;
        Ok(response)
    }
}

impl ServiceConfig {
    fn get_updated_request(
        &self,
        rest: &str,
        req: &mut Request<Body>,
    ) -> Result<(), Box<dyn Error>> {
        let uri = Url::from_str(&self.url.clone())?;
        let uri = uri.join(rest)?;
        let params: Vec<(String, String)> = self
            .query_params
            .iter()
            .filter(|x| x.action == Action::Add)
            .map(|x| (x.key.clone(), x.value.clone()))
            .collect();
        let uri: Url = Url::parse_with_params(&uri.to_string(), &params)?;
        *req.uri_mut() = Uri::from_str(&uri.to_string())?;
        let headers_mut = req.headers_mut();
        for header in &self.headers {
            if header.action == Action::Add {
                let key = HeaderName::from_str(&header.key)?;
                headers_mut.append(key, HeaderValue::from_str(&header.value)?);
            }
        }
        for header in HOP_HEADERS.iter() {
            headers_mut.remove(header);
        }
        Ok(())
    }
}

#[derive(Clone)]
struct SimpleProject<'a> {
    name: &'a str,
    services: DashMap<String, (ServiceConfig, Box<dyn ProxyService>)>,
}

#[async_trait]
impl<'a> Project for SimpleProject<'a> {
    async fn is_project(&self, path: &str) -> bool {
        return self.name == path;
    }

    async fn get_service(
        &self,
        path: String,
    ) -> Option<RefMut<'async_trait, String, (ServiceConfig, Box<dyn ProxyService>)>> {
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
        println!("project is {} and service is {}", project, service);
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

pub fn simple_project_handler() -> SimpleProjectHandler {
    let service_config = ServiceConfig {
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
        url: "http://httpbin.org/".to_string(),
        handler: ProxyParams {
            params: json! ({
                "username":"prasanth",
                "password": "password"
            }),
            handler_type: "basic_auth".to_string(),
        },
    };

    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let service = ServiceBuilder::new()
        .layer(BasicAuthLayer::new(
            "prasanth".to_string(),
            "prasanth".to_string(),
        ))
        .service(client);
    let basic_auth: Box<dyn ProxyService> = Box::new(service);
    let basic_auth_config: (ServiceConfig, Box<dyn ProxyService>) = (service_config, basic_auth);
    // let header_service_config = ServiceConfig {
    //     method: crate::config::Method::ANY,
    //     query_params: vec![UrlParam {
    //         key: "test".to_string(),
    //         value: "test".to_string(),
    //         action: Action::Add,
    //     }],
    //     headers: vec![Header {
    //         key: "test".to_string(),
    //         value: "test".to_string(),
    //         action: Action::Add,
    //     }],
    //     url: "http://httpbin.org/get".to_string(),
    //     handler: ProxyParams {
    //         params: json! ({
    //             "key":"rama",
    //             "value": "ranga"
    //         }),
    //         handler_type: "header_auth".to_string(),
    //     },
    // };

    let service_map = DashMap::new();
    service_map.insert("sample1".to_string(), basic_auth_config);
    let map: DashMap<String, Box<dyn Project>> = DashMap::new();
    map.insert(
        "first".to_string(),
        Box::new(SimpleProject {
            name: "first",
            services: service_map,
        }),
    );
    SimpleProjectHandler { projects: map }
}
