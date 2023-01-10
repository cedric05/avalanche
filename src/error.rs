use std::{error::Error, fmt::Display};

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum MarsError {
    UrlError(String),
    ServiceConfigError(String),
    ServiceNotRegistered,
    Error(Box<dyn Error + Sync + Send>),
}

impl Display for MarsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarsError::UrlError(str) => writeln!(f, "UrlError {}", str),
            MarsError::ServiceConfigError(str) => writeln!(f, "ServiceConfigError {}", str),
            MarsError::Error(error) => writeln!(f, "dynamic error error {}", error),
            MarsError::ServiceNotRegistered => writeln!(f, "service not registered error"),
        }
    }
}

impl std::error::Error for MarsError {}
