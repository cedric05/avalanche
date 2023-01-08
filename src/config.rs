use std::{error::Error, str::FromStr};

use http::{header::HeaderName, HeaderValue, Request, Uri};
use hyper::Body;
use serde::{Deserialize, Serialize};

use url::Url;

use crate::{error::MarsError, project::AVALANCHE_TOKEN};
pub use mars_config::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub url: String,
    pub method: Method,
    pub query_params: Vec<UrlParam>,
    pub headers: Vec<Header>,
    pub handler: ProxyParams,
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
    pub fn get_handler_config(&self, key: &str) -> Result<&str, MarsError> {
        self.handler
            .params
            .get(key)
            .ok_or(MarsError::ServiceConfigError)?
            .as_str()
            .ok_or(MarsError::ServiceConfigError)
    }

    pub fn get_timeout(&self) -> Option<f64> {
        self.handler.params.get("timeout").and_then(|x| x.as_f64())
    }

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
        let uri: Url = Url::parse_with_params(uri.as_ref(), &params)?;
        *req.uri_mut() = Uri::from_str(uri.as_ref())?;
        let headers_mut = req.headers_mut();
        headers_mut.remove(AVALANCHE_TOKEN);
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
