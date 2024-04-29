/// This module contains the implementation of the database-related functionality for the Mars Rover application.
/// It includes structs for managing projects and services, as well as functions for retrieving project managers and database connections.
use crate::{
    auth::{get_auth_service, ProxyService},
    project::AuthToken,
};
use dashmap::{mapref::one::RefMut, DashMap};
use mars_entity::user;
use sea_orm::{ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter};

use std::sync::Arc;
use std::{error::Error, str::FromStr};

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
                Some(subproject) => {
                    let config = ServiceConfig {
                        url: subproject.url,
                        method: subproject.method.0,
                        query_params: subproject.query_params.0,
                        headers: subproject.headers.0,
                        auth: subproject.auth.0,
                        params: subproject.params.0,
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

    async fn exists(&self, token: &AuthToken, project_index: &str) -> bool {
        use mars_entity::authtoken;
        use mars_entity::project;
        use mars_entity::user_project;

        let (token_type, auth_token) = match token.0.split_once(":") {
            Some((token_type, auth_token)) => (token_type, auth_token),
            None => return false,
        };
        let auth_token = match uuid::Uuid::from_str(auth_token) {
            Ok(uuid) => uuid,
            _ => return false,
        };
        match token_type {
            "user" => {
                // SELECT "user_projects"."id", "user_projects"."user_id", "user_projects"."project_id", "user_projects"."permissions" FROM "user_projects" INNER JOIN "project" ON "user_projects"."project_id" = "project"."id" INNER JOIN "users" ON "user_projects"."user_id" = "users"."id" INNER JOIN "authtoken" ON "authtoken"."user_id" = "user_projects"."id" WHERE "project"."index" = 'aviko' AND "authtoken"."auth_token" = '751d5169-204b-4c0f-9cbe-02abc6f5deb8';
                match user_project::Entity::find()
                    .filter(project::Column::Index.eq(project_index))
                    .inner_join(project::Entity)
                    .inner_join(user::Entity)
                    .inner_join(authtoken::Entity)
                    .filter(authtoken::Column::AuthToken.eq(auth_token))
                    .one(&self.db_conn)
                    .await
                {
                    Ok(Some(user_project_obj)) => {
                        println!("user_project_obj is {:?}", user_project_obj);
                        let execute = mars_entity::authtoken::AuthTokenPermissions::Execute as u8;
                        (user_project_obj.permissions as u8 & execute) == execute
                    }
                    Ok(None) => {
                        log::error!("no authtoken present in db");
                        false
                    }
                    Err(err) => {
                        log::error!("unable get data {}", err);
                        false
                    }
                }
            }
            "project" => {
                match authtoken::Entity::find()
                    .filter(authtoken::Column::AuthToken.eq(auth_token))
                    .inner_join(project::Entity)
                    .filter(project::Column::Index.eq(project_index))
                    .one(&self.db_conn)
                    .await
                {
                    Ok(Some(auth_token_obj)) => {
                        let execute = mars_entity::authtoken::AuthTokenPermissions::Execute as u8;
                        (auth_token_obj.permissions as u8 & execute) == execute
                    }
                    Ok(None) => {
                        log::error!("no authtoken present in db");
                        false
                    }
                    Err(err) => {
                        log::error!("unable get data {}", err);
                        false
                    }
                }
            }
            _ => false,
        }
    }
}

/// Retrieves a project manager for the database connection.
pub async fn get_db_project_manager(
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
    use super::DbProjectManager;
    use crate::project::ProjectManager;
    use dashmap::DashMap;
    use sea_orm::Database;

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
