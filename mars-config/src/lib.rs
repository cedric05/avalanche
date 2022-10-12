use serde::{Deserialize, Serialize};

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

pub struct ProxyParams {
    pub params: serde_json::Value,
    pub handler_type: String,
}
