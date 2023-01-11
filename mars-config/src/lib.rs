use serde::{Deserialize, Serialize};
use serde_json::json;

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
    pub params: serde_json::Value,
    pub auth_type: AuthType,
}

impl Default for MarsAuth {
    fn default() -> Self {
        Self {
            params: json!({}),
            auth_type: AuthType::NoAuth,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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

pub struct GeneralParams(pub serde_json::Value);

impl Default for GeneralParams {
    fn default() -> Self {
        Self(json!({}))
    }
}
