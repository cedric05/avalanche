use std::{error::Error, str::FromStr};

use http::{header::HeaderName, HeaderValue, Request, Uri};
use hyper::Body;
use serde::{Deserialize, Serialize};

use url::Url;

#[allow(unused)]
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
pub struct ServiceConfig {
    pub url: String,
    pub method: Method,
    pub query_params: Vec<UrlParam>,
    pub headers: Vec<Header>,
    pub handler: ProxyParams,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]

pub struct ProxyParams {
    pub params: serde_json::Value,
    pub handler_type: String,
}

lazy_static::lazy_static! {
    static ref HOP_HEADERS: Vec<HeaderName> = vec![
        HeaderName::from_str("Connection").unwrap(),
        HeaderName::from_str("Keep-Alive").unwrap(),
        HeaderName::from_str("Proxy-Authenticate").unwrap(),
        HeaderName::from_str("Proxy-Authorization").unwrap(),
        HeaderName::from_str("Te").unwrap(),
        HeaderName::from_str("Trailers").unwrap(),
        HeaderName::from_str("Transfer-Encoding").unwrap(),
        HeaderName::from_str("Upgrade").unwrap(),
        HeaderName::from_str("Host").unwrap(),
    ];
}
impl ServiceConfig {
    pub fn get_updated_request(
        &self,
        rest: &str,
        req: &mut Request<Body>,
    ) -> Result<(), Box<dyn Error>> {
        let uri = Url::from_str(&self.url.clone())?;
        let uri = uri.join(rest)?;
        let params: Vec<(String, String)> = self
            .query_params
            .iter()
            .filter(|x| x.action == Action::Add)
            .map(|x| (x.key.clone(), x.value.clone()))
            .collect();
        let uri: Url = Url::parse_with_params(&uri.to_string(), &params)?;
        *req.uri_mut() = Uri::from_str(&uri.to_string())?;
        let headers_mut = req.headers_mut();
        for header in &self.headers {
            if header.action == Action::Add {
                let key = HeaderName::from_str(&header.key)?;
                headers_mut.append(key, HeaderValue::from_str(&header.value)?);
            }
        }
        for header in HOP_HEADERS.iter() {
            headers_mut.remove(header);
        }
        Ok(())
    }
}
