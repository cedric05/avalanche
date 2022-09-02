#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Action {
    Add,
    Discard,
    Pass,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Header {
    key: String,
    value: String,
    action: Action,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct UrlParam {
    key: String,
    value: String,
    action: Action,
}

#[derive(Clone, Debug, Eq, PartialEq, Copy, Hash)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceConfig {
    pub url: String,
    pub method: Method,
    pub query_params: Vec<UrlParam>,
    pub headers: Vec<Header>,
    pub handler: ProxyParams,
}

#[derive(Clone, Debug, Eq, PartialEq)]

pub struct ProxyParams {
    pub params: serde_json::Value,
    pub handler_type: String,
}
