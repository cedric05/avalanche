use mars_rover::{db, file as json_project_manager, project::ProjectManager};
use clap::Parser;
/// This module contains the command-line interface (CLI) functionality for the Mars Rover project.
/// It defines the `Args` struct which represents the command-line arguments and provides methods to retrieve a project manager.
use std::{net::SocketAddr, sync::Arc};

/// Represents the command-line arguments for the Mars Rover project.
#[derive(Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub(crate) subcommand: DbParams,
    /// The address to bind the server to. Default value is "127.0.0.1:3000".
    #[clap(short, long, default_value = "127.0.0.1:3000")]
    pub(crate) addr: String,
}


#[derive(clap::Subcommand, Clone, Debug)]
pub enum DbParams{
    File{
        /// The path to the configuration file. Default value is "config/config.json5".
        #[clap(short, long, default_value = "config/config.json5")]
        config: String,
        tokens: Option<String>,
    },
    /// The optional database URL. Only available when the "sql" feature is enabled.
    #[cfg(feature = "sql")]
    Db {
        #[clap(short, long)]
        url: String,
    }
}



impl Args {
    /// Retrieves the project manager based on the command-line arguments.
    /// Returns an `Arc<Box<dyn ProjectManager>>`.
    pub async fn get_project_manager(&self) -> Arc<Box<dyn ProjectManager>> {
        match &self.subcommand {
            DbParams::File { config, tokens } => {
                json_project_manager::get_file_project_manager(config.clone().into(), tokens.clone().into())
                .await
                .expect("unable to load config")
            },
            #[cfg(feature = "sql")]
            DbParams::Db { url } => {
                db::get_db_project_manager(url)
                .await
                .expect("unable to connect to db")
            },
        }
    }

    pub fn get_addr(&self) -> SocketAddr {
        let port_key = "FUNCTIONS_CUSTOMHANDLER_PORT";
        match std::env::var(port_key) {
            Ok(val) => {
                let port = val.parse().expect("Custom Handler port is not a number!");
                std::net::SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, port))
            }
            Err(_) => <std::net::SocketAddr as std::str::FromStr>::from_str(&self.addr)
                .expect("Unable to parse address from cli"),
        }
    }
}
