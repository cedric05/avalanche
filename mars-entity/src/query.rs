use super::impl_mandatory_types;

pub use mars_config::{Action, Header, UrlParam as QueryParam};
use sea_orm::sea_query::ValueType;
use sea_orm::{self, Value};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryParams(pub Vec<QueryParam>);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Headers(pub Vec<Header>);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]

pub struct Auth(pub mars_config::MarsAuth);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]

pub struct GeneralParams(pub mars_config::GeneralParams);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]

pub struct Method(pub mars_config::Method);

impl_mandatory_types!(QueryParams);
impl_mandatory_types!(Headers);
impl_mandatory_types!(Auth);
impl_mandatory_types!(Method);
impl_mandatory_types!(GeneralParams);
