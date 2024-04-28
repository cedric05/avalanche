pub use mars_config::{Action, Header, UrlParam as QueryParam};
use sea_orm::{self, FromJsonQueryResult};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct QueryParams(pub Vec<QueryParam>);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct Headers(pub Vec<Header>);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]

pub struct Auth(pub mars_config::MarsAuth);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]

pub struct GeneralParams(pub mars_config::GeneralParams);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]

pub struct Method(pub mars_config::Method);