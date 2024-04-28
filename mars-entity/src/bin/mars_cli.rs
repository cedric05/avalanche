use std::collections::HashMap;

use clap::{Parser, Subcommand};
use mars_config::ServiceConfig;
use mars_entity::project::ActiveModel;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, ConnectionTrait, Schema, sea_query::TableCreateStatement};
use serde::{Deserialize, Serialize};
use mars_entity::project::Entity as ProjectEntity;
use mars_entity::subproject::Entity as SubProjectEntity;

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    action: SubCommand,

    #[clap(short, long, default_value = "config/config.json5")]
    file: String,

    #[clap(short, long)]
    db: String,
}

#[derive(Subcommand)]
enum SubCommand {
    Load,
    Dump,
    Orm,
}

#[derive(Serialize, Deserialize, Debug)]
struct MultipleProjects(HashMap<String, ProjectDTO>);

#[derive(Serialize, Deserialize, Debug)]
struct ProjectDTO {
    subprojects: HashMap<String, ServiceConfig>,
    needs_auth: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("able to parse file");
    let db = sea_orm::Database::connect(args.db)
        .await
        .expect("unable to connect to db");
    println!("able to connect to db");

    match args.action {
        SubCommand::Dump => {
            let mut living_projects = MultipleProjects(Default::default());
            for project in mars_entity::project::Entity::find()
                .all(&db)
                .await
                .expect("unable to make query")
            {
                let mut current_project = ProjectDTO {
                    subprojects: Default::default(),
                    needs_auth: project.needs_auth,
                };
                for service in mars_entity::subproject::Entity::find()
                    .filter(mars_entity::subproject::Column::ProjectId.eq(project.id))
                    .all(&db)
                    .await
                    .expect("unable to make query")
                {
                    let service_config = ServiceConfig {
                        url: service.url,
                        method: service.method.0,
                        query_params: service.query_params.0,
                        headers: service.headers.0,
                        auth: service.auth.0,
                        params: service.params.0,
                    };
                    current_project
                        .subprojects
                        .insert(service.index, service_config);
                }
                living_projects.0.insert(project.index, current_project);
            }

            println!("config of  projects and its services  \n\nXXXXXXXXXXXXXXXXXXXXXXX\n\n");

            match std::fs::OpenOptions::new()
                .create(true)
                .create_new(true)
                .write(true)
                .open(args.file)
            {
                Err(_out) => {
                    println!(
                        "failed to write to file {_out}, dumping here \n\n{}",
                        serde_json::to_string_pretty(&living_projects).expect("impossible to fail")
                    );

                    println!(
                        "\n\nXXXXXXXXXXXXXXXXXXXXXXX\n\n
                        done"
                    );
                }
                Ok(file) => serde_json::to_writer_pretty(file, &living_projects)
                    .expect("impossible to fail"),
            };
        }
        SubCommand::Load => {
            let config = std::fs::read_to_string(args.file).expect("unable to open file");
            println!("able to read file");
            let config: MultipleProjects = json5::from_str(&config).expect("unable to parse");
            let mut failed = MultipleProjects(Default::default());

            for (index, project) in config.0.into_iter() {
                let mut failed_project = ProjectDTO {
                    subprojects: Default::default(),
                    needs_auth: project.needs_auth,
                };
                let project_id = match mars_entity::project::Entity::find()
                    .filter(mars_entity::project::Column::Index.eq(index.clone()))
                    .one(&db)
                    .await
                    .expect("unable to make query")
                {
                    Some(exists) => exists.id,
                    None => {
                        let proect_active_model = ActiveModel {
                            id: sea_orm::ActiveValue::NotSet,
                            index: sea_orm::ActiveValue::Set(index.clone()),
                            needs_auth: sea_orm::ActiveValue::Set(project.needs_auth),
                        };
                        let res = mars_entity::project::Entity::insert(proect_active_model)
                            .exec(&db)
                            .await
                            .expect("unable to insert");
                        res.last_insert_id
                    }
                };

                for (service_index, service_config) in project.subprojects.into_iter() {
                    match mars_entity::subproject::Entity::find()
                        .filter(mars_entity::subproject::Column::Index.eq(service_index.clone()))
                        .filter(mars_entity::subproject::Column::ProjectId.eq(project_id))
                        .one(&db)
                        .await
                        .expect("unable to make query")
                    {
                        Some(_exists) => {
                            println!("for project {index} subproject {service_index} already exists, not updating it");
                            failed_project
                                .subprojects
                                .insert(service_index.clone(), service_config);
                        }
                        None => {
                            let pear = mars_entity::subproject::ActiveModel {
                                project_id: sea_orm::ActiveValue::Set(project_id),
                                id: sea_orm::ActiveValue::NotSet,
                                method: sea_orm::ActiveValue::Set(mars_entity::subproject::Method(
                                    service_config.method,
                                )),
                                query_params: sea_orm::ActiveValue::Set(
                                    mars_entity::subproject::QueryParams(
                                        service_config.query_params,
                                    ),
                                ),
                                headers: sea_orm::ActiveValue::Set(
                                    mars_entity::subproject::Headers(service_config.headers),
                                ),
                                auth: sea_orm::ActiveValue::Set(mars_entity::subproject::Auth(
                                    service_config.auth,
                                )),
                                params: sea_orm::ActiveValue::Set(
                                    mars_entity::subproject::GeneralParams(service_config.params),
                                ),
                                index: sea_orm::ActiveValue::Set(service_index.clone()),
                                url: sea_orm::ActiveValue::Set(service_config.url),
                            };
                            if let Err(err) = mars_entity::subproject::Entity::insert(pear)
                                .exec(&db)
                                .await
                            {
                                println!(
                                    "unable to insert sub project {service_index} for project {index}, error {err}"
                                );
                            }
                        }
                    };
                }
                if !failed_project.subprojects.is_empty() {
                    failed.0.insert(index, failed_project);
                }
            }
            if !failed.0.is_empty() {
                println!(
                    "below is config of all failed services with project wise \n\nXXXXXXXXXXXXXXXXXXXXXXX\n\n"
                );

                println!(
                    "{}",
                    serde_json::to_string_pretty(&failed).expect("impossible to fail")
                );

                println!(
                    "\n\nXXXXXXXXXXXXXXXXXXXXXXX\n\n
                    done"
                );
            }
        }
        SubCommand::Orm=>{
            let builder = db.get_database_backend();
            let schema = Schema::new(builder);
            // Derive from Entity
            let stmt: TableCreateStatement = schema.create_table_from_entity(ProjectEntity);
            let result = db.execute(db.get_database_backend().build(&stmt)).await;

            println!("created project {:?}", result);

            let stmt: TableCreateStatement = schema.create_table_from_entity(SubProjectEntity);
            let result = db.execute(db.get_database_backend().build(&stmt)).await;
            println!("created subproject {:?}", result);
        }
    }
}
