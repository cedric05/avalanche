/// This module contains the implementation of the database-related functionality for the Mars Rover application.
/// It includes structs for managing projects and services, as well as functions for retrieving project managers and database connections.


use crate::auth::{get_auth_service, ProxyService};
use dashmap::{mapref::one::RefMut, DashMap};
use sea_orm::{ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter};

use std::error::Error;
use std::sync::Arc;

use crate::project::{AuthProjectRequestHandler, ProjectManager};
use mars_config::{MarsError, ServiceConfig};

/// Represents a project in the database.
#[derive(Clone)]
pub(crate) struct DbProject {
    name: String,
    project_id: i32,
    services: DashMap<String, ProxyService>,
    needs_auth: bool,
    db_con: DatabaseConnection,
}

#[async_trait::async_trait]
impl AuthProjectRequestHandler for DbProject {
    /// Checks if the given path matches the name of the project.
    async fn is_project(&self, path: &str) -> bool {
        self.name == path
    }

    /// Retrieves the service associated with the given path.
    async fn get_service(
        &self,
        path: String,
    ) -> Result<Option<RefMut<String, ProxyService>>, Box<dyn Error>> {
        if self.services.contains_key(&path) {
            Ok(self.services.get_mut(&path))
        } else {
            use mars_entity::subproject;
            match subproject::Entity::find()
                .filter(subproject::Column::ProjectId.eq(self.project_id))
                .filter(subproject::Column::Index.eq(path.clone()))
                .one(&self.db_con)
                .await?
            {
                Some(project) => {
                    let config = ServiceConfig {
                        url: project.url,
                        method: project.method.0,
                        query_params: project.query_params.0,
                        headers: project.headers.0,
                        auth: project.auth.0,
                        params: project.params.0,
                    };
                    println!("config is {:?}", config);
                    match get_auth_service(config) {
                        Ok(res) => {
                            self.services.insert(path.clone(), res);
                            Ok(self.services.get_mut(&path))
                        }
                        Err(err) => Err(Box::new(MarsError::ServiceConfigError(format!(
                            "unable to derive auth config error: `{}`",
                            err
                        )))),
                    }
                }
                None => Err(Box::new(MarsError::ServiceConfigError(format!(
                    "for project `{}` service: `{}` is not configured",
                    self.project_id, path,
                )))),
            }
        }
    }

    /// Checks if authentication is configured for the project.
    async fn auth_configured(&self) -> bool {
        self.needs_auth
    }
}

/// Represents a project manager that interacts with the database.
#[derive(Clone)]
pub(crate) struct DbProjectManager {
    db_conn: DatabaseConnection,
    projects: DashMap<String, Arc<Box<dyn AuthProjectRequestHandler>>>,
}

#[async_trait::async_trait]
impl ProjectManager for DbProjectManager {
    /// Retrieves the project with the given project key.
    async fn get_project(
        &self,
        project_key: String,
    ) -> Result<Option<Arc<Box<dyn AuthProjectRequestHandler>>>, Box<dyn Error>> {
        match self.projects.get_mut(&project_key) {
            Some(service) => Ok(Some(service.clone())),
            None => {
                use mars_entity::project;
                match project::Entity::find()
                    .filter(project::Column::Index.eq(project_key.clone()))
                    .one(&self.db_conn)
                    .await?
                {
                    Some(project) => {
                        let db_project = DbProject {
                            name: project_key.clone(),
                            project_id: project.id,
                            services: DashMap::default(),
                            needs_auth: project.needs_auth,
                            db_con: self.db_conn.clone(),
                        };
                        self.projects
                            .insert(project_key.clone(), Arc::new(Box::new(db_project)));
                        self.get_project(project_key).await
                    }
                    None => Ok(None),
                }
            }
        }
    }
}

/// Retrieves a project manager for the database connection.
pub(crate) async fn get_db_project_manager(
    url: &str,
) -> Result<Arc<Box<dyn ProjectManager>>, Box<dyn Error>> {
    let db = Database::connect(url).await?;

    let project_manager = DbProjectManager {
        db_conn: db,
        projects: DashMap::default(),
    };
    Ok(Arc::new(Box::new(project_manager)))
}

#[cfg(test)]
mod test {
    use dashmap::DashMap;
    use sea_orm::Database;
    use crate::project::ProjectManager;
    use super::DbProjectManager;

    #[ignore]
    #[tokio::test]
    async fn test_basic() {
        let db = Database::connect(
            "sqlite:///home/neptune/projects/personal/cedric05/mars-rover/db.sqlite",
        )
        .await
        .unwrap();

        let project_manager = DbProjectManager {
            db_conn: db,
            projects: DashMap::default(),
        };
        let project = project_manager.get_project("test".to_string()).await;
        match project {
            Ok(Some(project)) => {
                println!("project fetch working");
                let service = project.get_service("test2".to_string()).await;
                match service {
                    Ok(Some(_service)) => {
                        println!("service fetch working  ");
                    }
                    Ok(None) => {
                        println!("service not available in db ");
                    }
                    Err(error) => {
                        println!("service fetching ran into error {:?}", error);
                    }
                }
            }
            Ok(None) => {
                println!("project not available in db");
            }
            Err(error) => {
                println!("project ran into error {:?}", error);
            }
        }
    }
}
