use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

pub enum AuthTokenPermissions {
    Read = 0b1,
    Write = 0b10,
    Execute = 0b100,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "authtoken")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub project_id: Option<i32>,
    pub user_id: Option<i32>,
    pub auth_token: Uuid,
    pub permissions: i32,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Project,
    User,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Project => Entity::belongs_to(super::project::Entity)
                .from(Column::ProjectId)
                .to(super::project::Column::Id)
                .into(),
            Self::User => Entity::belongs_to(super::user::Entity)
                .from(Column::UserId)
                .to(super::user::Column::Id)
                .into(),
        }
    }
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
