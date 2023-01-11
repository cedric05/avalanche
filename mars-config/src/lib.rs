use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
mod config;
mod error;

pub use config::ServiceConfig;
pub use error::*;

pub const AVALANCHE_TOKEN: &str = "avalanche-token";

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Action {
    Add,
    Discard,
    Pass,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Header {
    pub key: String,
    pub value: String,
    pub action: Action,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct UrlParam {
    pub key: String,
    pub value: String,
    pub action: Action,
}

#[allow(unused)]
#[derive(Clone, Debug, Eq, PartialEq, Copy, Hash, Serialize, Deserialize)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    OPTIONS,
    CONNECT,
    HEAD,
    TRACE,
    PATCH,
    COPY,
    LINK,
    UNLINK,
    PURGE,
    LOCK,
    UNLOCK,
    PROPFIND,
    VIEW,
    MKCOL,
    MOVE,
    PROPPATCH,
    REPORT,
    SEARCH,
    ANY,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]

pub struct MarsAuth {
    params: serde_json::Value,
    auth_type: AuthType,
}

impl MarsAuth {
    pub fn get_param(&self, key: &str) -> Option<&Value> {
        self.params.get(key)
    }

    pub fn get_params(&self) -> Value {
        self.params.clone()
    }

    pub fn auth_type(&self) -> &AuthType {
        &self.auth_type
    }

    pub fn new(params: serde_json::Value, auth_type: AuthType) -> Self {
        Self { params, auth_type }
    }
}

impl Default for MarsAuth {
    fn default() -> Self {
        Self {
            params: json!({}),
            auth_type: AuthType::NoAuth,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Copy)]
#[non_exhaustive]
pub enum AuthType {
    #[serde(rename = "basic_auth")]
    BasicAuth,
    #[serde(rename = "header_auth")]
    HeaderAuth,
    #[serde(rename = "aws_auth")]
    AwsAuth,
    #[serde(rename = "x509")]
    X509Auth,
    #[serde(rename = "hawk_auth")]
    HawkAuth,
    #[serde(rename = "digest_auth")]
    DigestAuth,
    #[serde(rename = "no_auth")]
    NoAuth,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]

pub struct GeneralParams(serde_json::Value);

impl GeneralParams {
    pub fn get_value(&self, key: &str) -> Option<&serde_json::Value> {
        self.0.get(key)
    }

    pub fn new(value: serde_json::Value) -> Self {
        GeneralParams(value)
    }
}

impl Default for GeneralParams {
    fn default() -> Self {
        Self(json!({}))
    }
}
