use crate::auth::{get_auth_service, ProxyService};

use async_trait::async_trait;
use dashmap::{mapref::one::RefMut, DashMap};

use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use std::{convert::TryFrom, error::Error};

use crate::project::{AuthProjectRequestHandler, ProjectManager};
use mars_config::{MarsError, ServiceConfig};


/// `FileBasedProject` represents a project that is configured based on a file.
///
/// It contains a map of service configurations, a flag indicating whether authentication is needed,
/// a name, and a map of services. The `try_from` method is used to create an instance of `FileBasedProject`
/// from a JSON configuration.
#[derive(Clone)]
struct FileBasedProject {
    name: String,
    service_config_map: HashMap<String, ServiceConfig>,
    services: DashMap<String, ProxyService>,
    needs_auth: bool,
}

#[async_trait]
impl AuthProjectRequestHandler for FileBasedProject {
    async fn is_project(&self, path: &str) -> bool {
        return self.name == path;
    }

    async fn auth_configured(&self) -> bool {
        self.needs_auth
    }

    async fn get_service(
        &self,
        path: String,
    ) -> Result<Option<RefMut<String, ProxyService>>, Box<dyn Error>> {
        if self.services.contains_key(&path) {
            Ok(self.services.get_mut(&path))
        } else if let Some(config) = self.service_config_map.get(&path).cloned() {
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



/// `FileProjectManager` is responsible for managing `FileBasedProject`s.
///
/// It contains a map of projects, where each project is an instance of a type that implements the
/// `AuthProjectRequestHandler` trait. The `try_from` method is used to create an instance of `FileProjectManager`
/// from a JSON configuration.
#[derive(Clone)]
pub(crate) struct FileProjectManager {
    projects: DashMap<String, Arc<Box<dyn AuthProjectRequestHandler>>>,
}

// unsafe impl Send for SimpleProjectHandler {}

#[async_trait]
impl ProjectManager for FileProjectManager {
    async fn get_project(
        &self,
        project_key: String,
    ) -> Result<Option<Arc<Box<dyn AuthProjectRequestHandler>>>, Box<dyn Error>> {
        let project = self.projects.get_mut(&project_key).ok_or_else(|| {
            MarsError::ServiceConfigError(format!("project `{project_key}` is missing"))
        })?;
        Ok(Some(project.clone()))
    }
}

impl TryFrom<Value> for FileBasedProject {
    type Error = MarsError;

    fn try_from(mut project_config: Value) -> Result<Self, Self::Error> {
        let project_config = project_config
            .as_object_mut()
            .ok_or_else(|| MarsError::ServiceConfigError("project is not object".to_string()))?;

        let needs_auth = project_config
            .get("needs_auth")
            .unwrap_or(&Value::Bool(true))
            .as_bool()
            .unwrap_or(true);
        let service_map = DashMap::new();
        let mut service_config_map = HashMap::new();
        let sub_project_config = project_config
            .get_mut("subprojects")
            .ok_or_else(|| {
                MarsError::ServiceConfigError("subprojects is not available".to_string())
            })?
            .as_object_mut()
            .ok_or_else(|| {
                MarsError::ServiceConfigError("subprojects is not object".to_string())
            })?;

        for (service_key, service_config_value) in sub_project_config.into_iter() {
            let service_config = service_config_value.take();
            let service_config: ServiceConfig =
                serde_json::from_value(service_config).map_err(|_| {
                    MarsError::ServiceConfigError(format!(
                        "serviceconfig for `{service_key}` is not parsable "
                    ))
                })?;
            service_config_map.insert(service_key.to_string(), service_config);
        }
        Ok(FileBasedProject {
            service_config_map,
            needs_auth,
            name: "no meaning as of now".to_string(),
            services: service_map,
        })
    }
}

impl TryFrom<Value> for FileProjectManager {
    type Error = MarsError;

    fn try_from(mut value: Value) -> Result<Self, Self::Error> {
        let all_config = value
            .as_object_mut()
            .ok_or_else(|| MarsError::ServiceConfigError("config is not object".to_string()))?;
        let map = DashMap::new();
        for (project_key, project_config) in all_config {
            let project_config = project_config.take();
            let project = FileBasedProject::try_from(project_config)?;
            let project: Box<dyn AuthProjectRequestHandler> = Box::new(project);
            map.insert(project_key.to_string(), Arc::new(project));
        }
        Ok(FileProjectManager { projects: map })
    }
}

pub(crate) fn get_file_project_manager(
    path: PathBuf,
) -> Result<Arc<Box<dyn ProjectManager>>, MarsError> {
    let mut file = fs::File::open(path)
        .map_err(|err| MarsError::ServiceConfigError(format!("ran into error {}", err)))?;
    let mut config = String::new();
    file.read_to_string(&mut config)
        .map_err(|err| MarsError::ServiceConfigError(format!("ran into error {}", err)))?;
    let value: Value = json5::from_str(&config)
        .map_err(|err| MarsError::ServiceConfigError(format!("ran into error {}", err)))?;
    Ok(Arc::new(Box::new(FileProjectManager::try_from(value)?)))
}
