use std::{error::Error, fmt::Display};

#[allow(unused)]
#[derive(Debug)]
pub enum MarsError {
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

pub trait ToMarsError<T, E>
where
    T: Sized,
    E: Sized,
{
    fn config_error(self, message: String) -> core::result::Result<E, MarsError>;
}

impl<T> ToMarsError<T, T> for Option<T> {
    fn config_error(self, message: String) -> core::result::Result<T, MarsError> {
        self.ok_or_else(|| MarsError::ServiceConfigError(message))
    }
}
