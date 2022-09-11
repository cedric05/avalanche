use std::error::Error;

use async_trait::async_trait;
use dashmap::{mapref::one::RefMut, DashMap};
use http::Response;
use hyper::{Body, Client};
use hyper_tls::HttpsConnector;
use serde_json::json;
use tower::ServiceBuilder;

use crate::{
    basicauth::BasicAuthLayer,
    config::{Action, Header, ProxyParams, ServiceConfig, UrlParam},
    error::MarsError,
    project::{Project, ProjectHandler, ProxyService},
};

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
        .layer(BasicAuthLayer::from_username_n_password("prasanth", "prasanth").unwrap())
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
