use crate::auth::get_auth_service;
use crate::error::MarsError;

use async_trait::async_trait;
use dashmap::{mapref::one::RefMut, DashMap};

use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::{convert::TryFrom, error::Error};

use crate::config::ServiceConfig;
use crate::project::{ProjectHandler, ProjectManager, ProxyService};

#[derive(Clone)]
struct SimpleProject {
    name: String,
    service_config_map: HashMap<String, ServiceConfig>,
    services: DashMap<String, (ServiceConfig, Box<dyn ProxyService>)>,
    needs_auth: bool,
}

#[async_trait]
impl ProjectHandler for SimpleProject {
    async fn is_project(&self, path: &str) -> bool {
        return self.name == path;
    }

    async fn auth_configured(&self) -> bool {
        self.needs_auth
    }

    async fn get_service(
        &self,
        path: String,
    ) -> Result<Option<RefMut<String, (ServiceConfig, Box<dyn ProxyService>)>>, Box<dyn Error>>
    {
        if self.services.contains_key(&path) {
            Ok(self.services.get_mut(&path))
        } else {
            if let Some(config) = self.service_config_map.get(&path).cloned() {
                if let Ok(res) = get_auth_service(config) {
                    self.services.insert(path.clone(), res);
                    Ok(self.services.get_mut(&path))
                } else {
                    // TODO
                    // need to handle error scenarios
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        }
    }
}

#[derive(Clone)]
pub struct SimpleProjectManager {
    projects: DashMap<String, Box<dyn ProjectHandler>>,
}

// unsafe impl Send for SimpleProjectHandler {}

#[async_trait]
impl ProjectManager for SimpleProjectManager {
    async fn get_project(
        &self,
        project_key: String,
    ) -> Result<Option<Box<dyn ProjectHandler>>, Box<dyn Error>> {
        let project = self
            .projects
            .get_mut(&project_key)
            .ok_or(MarsError::ServiceConfigError)?;
        Ok(Some(project.clone()))
    }
}

impl TryFrom<Value> for SimpleProject {
    type Error = MarsError;

    fn try_from(mut project_config: Value) -> Result<Self, Self::Error> {
        let project_config = project_config
            .as_object_mut()
            .ok_or(MarsError::ServiceConfigError)?;

        let needs_auth = project_config
            .get("needs_auth")
            .unwrap_or(&Value::Bool(true))
            .as_bool()
            .unwrap_or(true);
        let service_map = DashMap::new();
        let mut service_config_map = HashMap::new();
        let sub_project_config = project_config
            .get_mut("subprojects")
            .ok_or(MarsError::ServiceConfigError)?
            .as_object_mut()
            .ok_or(MarsError::ServiceConfigError)?;

        for (service_key, service_config_value) in sub_project_config.into_iter() {
            let service_config = service_config_value.take();
            let service_config: ServiceConfig = serde_json::from_value(service_config)
                .map_err(|_| MarsError::ServiceConfigError)?;
            service_config_map.insert(service_key.to_string(), service_config);
        }
        Ok(SimpleProject {
            service_config_map,
            needs_auth: needs_auth,
            name: "no meaning as of now".to_string(),
            services: service_map,
        })
    }
}

impl TryFrom<Value> for SimpleProjectManager {
    type Error = MarsError;

    fn try_from(mut value: Value) -> Result<Self, Self::Error> {
        let all_config = value.as_object_mut().ok_or(MarsError::ServiceConfigError)?;
        let map = DashMap::new();
        for (project_key, project_config) in all_config {
            let project_config = project_config.take();
            let project = SimpleProject::try_from(project_config)?;
            let project: Box<dyn ProjectHandler> = Box::new(project);
            map.insert(project_key.to_string(), project);
        }
        Ok(SimpleProjectManager { projects: map })
    }
}

pub fn simple_project_handler(path: PathBuf) -> Result<SimpleProjectManager, MarsError> {
    let mut file = fs::File::open(path).map_err(|_| MarsError::ServiceConfigError)?;
    let mut config = String::new();
    file.read_to_string(&mut config)
        .map_err(|_| MarsError::ServiceConfigError)?;
    let value: Value = json5::from_str(&config).map_err(|_| MarsError::ServiceConfigError)?;
    SimpleProjectManager::try_from(value)
}
