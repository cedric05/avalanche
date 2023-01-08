use crate::auth::get_auth_service;

use dashmap::{mapref::one::RefMut, DashMap};
use sea_orm::{ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter};

use std::error::Error;
use std::sync::Arc;

use crate::config::ServiceConfig;
use crate::project::{ProjectHandler, ProjectManager, ProxyService};

#[derive(Clone)]
pub struct DbProject {
    name: String,
    project_id: i32,
    services: DashMap<String, (ServiceConfig, Box<dyn ProxyService>)>,
    needs_auth: bool,
    db_con: DatabaseConnection,
}

#[async_trait::async_trait]
impl ProjectHandler for DbProject {
    async fn is_project(&self, path: &str) -> bool {
        self.name == path
    }

    async fn get_service(
        &self,
        path: String,
    ) -> Result<Option<RefMut<String, (ServiceConfig, Box<dyn ProxyService>)>>, Box<dyn Error>>
    {
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
                        handler: project.handler_type.0,
                    };
                    println!("config is {:?}", config);
                    if let Ok(res) = get_auth_service(config) {
                        self.services.insert(path.clone(), res);
                        Ok(self.services.get_mut(&path))
                    } else {
                        // TODO
                        // need to handle error scenarios
                        Ok(None)
                    }
                }
                None => Ok(None),
            }
        }
    }

    async fn auth_configured(&self) -> bool {
        self.needs_auth
    }
}

#[derive(Clone)]
pub struct DbProjectManager {
    db_conn: DatabaseConnection,
    projects: DashMap<String, Arc<Box<dyn ProjectHandler>>>,
}

#[async_trait::async_trait]
impl ProjectManager for DbProjectManager {
    async fn get_project(
        &self,
        project_key: String,
    ) -> Result<Option<Arc<Box<dyn ProjectHandler>>>, Box<dyn Error>> {
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

pub async fn get_db_project_manager(
    url: &str,
) -> Result<Arc<Box<dyn ProjectManager>>, Box<dyn Error>> {
    let db = Database::connect(url).await.unwrap();

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

    #[tokio::test]
    async fn haha() {
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
                    Ok(Some(service)) => {
                        println!("service fetch working  ");
                    }
                    Ok(None) => {
                        println!("service not avaiabile in db ");
                    }
                    Err(error) => {
                        println!("service fetching ran into error {:?}", error);
                    }
                }
            }
            Ok(None) => {
                println!("project not avaiabile in db");
            }
            Err(error) => {
                println!("project ran into error {:?}", error);
            }
        }
    }
}
