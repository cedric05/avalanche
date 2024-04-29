use crate::{db, json_project_manager, project::ProjectManager};
use clap::Parser;
/// This module contains the command-line interface (CLI) functionality for the Mars Rover project.
/// It defines the `Args` struct which represents the command-line arguments and provides methods to retrieve a project manager.
use std::sync::Arc;

/// Represents the command-line arguments for the Mars Rover project.
#[derive(Parser)]
pub(crate) struct Args {
    /// The path to the configuration file. Default value is "config/config.json5".
    #[clap(short, long, default_value = "config/config.json5")]
    pub(crate) config: String,

    /// The optional database URL. Only available when the "sql" feature is enabled.
    #[cfg(feature = "sql")]
    #[clap(short, long)]
    pub(crate) db: Option<String>,

    /// The address to bind the server to. Default value is "127.0.0.1:3000".
    #[clap(short, long, default_value = "127.0.0.1:3000")]
    pub(crate) addr: String,
}

impl Args {
    /// Retrieves the project manager based on the command-line arguments.
    /// Returns an `Arc<Box<dyn ProjectManager>>`.
    pub async fn get_project_manager(&self) -> Arc<Box<dyn ProjectManager>> {
        #[cfg(feature = "sql")]
        if cfg!(feature = "sql") {
            return match &self.db {
                Some(db_url) => db::get_db_project_manager(db_url)
                    .await
                    .expect("unable to connect to db"),
                None => json_project_manager::get_file_project_manager(self.config.clone().into())
                    .await
                    .expect("unable to load config"),
            };
        }
        json_project_manager::get_file_project_manager(self.config.clone().into())
            .await
            .expect("unable to load config")
    }
}
