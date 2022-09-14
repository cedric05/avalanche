use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum MarsError {
    UrlError,
    ServiceConfigError,
    Error(Box<dyn Error + Sync + Send>),
}

impl Display for MarsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarsError::UrlError => f.write_str("urlerror"),
            MarsError::ServiceConfigError => f.write_str("service config error"),
            MarsError::Error(error) => writeln!(f, "dynamic error error {}", error),
        }
    }
}

impl std::error::Error for MarsError {}
