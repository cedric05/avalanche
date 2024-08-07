use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub url: String,
    pub method: Method,
    #[serde(default)]
    pub query_params: Vec<UrlParam>,
    #[serde(default)]
    pub headers: Vec<Header>,
    #[serde(default)]
    pub auth: MarsAuth,
    #[serde(default)]
    pub params: GeneralParams,
}

impl ServiceConfig {
    pub fn get_authparam_value_as_str(&self, key: &str) -> Option<&str> {
        self.auth.get_param(key).and_then(|x| x.as_str())
    }

    // timeout for a request
    pub fn get_timeout(&self) -> Option<f64> {
        self.params
            .get_value(crate::consts::TIMEOUT)
            .and_then(|x| x.as_f64())
    }

    pub fn get_params(&self, key: &str) -> Option<&Value> {
        self.params.get_value(key)
    }

    // allowed number of requests at a time
    pub fn get_concurrency_timeout(&self) -> Option<f64> {
        self.params
            .get_value(crate::consts::CONCURRENCY_LIMIT)
            .and_then(|x| x.as_f64())
    }

    // allowed number of requests for one second duration
    #[allow(unused)]
    pub fn get_rate_timeout(&self) -> Option<f64> {
        self.params
            .get_value(crate::consts::RATE_LIMIT)
            .and_then(|x| x.as_f64())
    }
}
