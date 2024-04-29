/// This module contains the definitions for Mars configuration.
/// It includes structs and enums for actions, headers, URL parameters,
/// authentication types, and general parameters.
///
/// # Examples
///
/// ```
/// use mars_config::*;
///
/// // Create a new MarsAuth instance with empty parameters and NoAuth authentication type
/// let auth = MarsAuth::default();
///
/// // Get a specific parameter value from MarsAuth
/// let param = auth.get_param("key");
///
/// // Get all parameters from MarsAuth
/// let params = auth.get_params();
///
/// // Get the authentication type from MarsAuth
/// let auth_type = auth.auth_type();
///
/// // Create a new GeneralParams instance with empty value
/// let general_params = GeneralParams::default();
///
/// // Get a specific value from GeneralParams
/// let value = general_params.get_value("key");
///
/// // Create a new AvalancheTrace instance with a string value
/// let trace = AvalancheTrace("trace".to_string());
/// ```
///
/// For more information, refer to the individual struct and enum documentation.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
mod config;
mod consts;
mod error;
pub use config::ServiceConfig;
pub use error::*;

pub use consts::*;

/// `Action` represents an action that can be performed by a user.
///
/// It contains fields for the action's name, description, and other relevant information.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Action {
    Add,
    Discard,
    Pass,
}

/// `Header` represents an HTTP header.
///
/// It contains fields for the header's name and value.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Header {
    pub key: String,
    pub value: String,
    pub action: Action,
}

/// `UrlParam` represents a URL parameter.
///
/// It contains fields for the parameter's name and value.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct UrlParam {
    pub key: String,
    pub value: String,
    pub action: Action,
}

/// `Method` represents an HTTP method.
///
/// It is an enum with variants for each possible HTTP method, such as GET, POST, PUT, DELETE, etc.
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

/// `MarsAuth` represents an authentication object for Mars.
///
/// It contains a `params` field, which is a JSON value that contains the authentication parameters,
/// and an `auth_type` field, which indicates the type of authentication.
///
/// The `get_param` method can be used to get a specific parameter from `params`, the `get_params` method
/// can be used to get a clone of `params`, and the `auth_type` method can be used to get a reference to `auth_type`.
///
/// The `new` method can be used to create a new instance of `MarsAuth`.
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Default, Deserialize)]
pub struct AvalancheTrace(pub String);
