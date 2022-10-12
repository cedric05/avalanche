use super::impl_mandatory_types;

pub use mars_config::{Action, Header, ProxyParams, UrlParam as QueryParam};
use sea_orm::sea_query::ValueType;
use sea_orm::{self, Value};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryParams(pub Vec<QueryParam>);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Headers(pub Vec<Header>);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]

pub struct HandlerParams(pub ProxyParams);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]

pub struct Method(pub mars_config::Method);

impl_mandatory_types!(QueryParams);
impl_mandatory_types!(Headers);
impl_mandatory_types!(HandlerParams);
impl_mandatory_types!(Method);
