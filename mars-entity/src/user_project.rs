use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_projects")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,
    pub project_id: i32,
    pub permissions: i32,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    User,
    Project,
    AuthToken,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::User => Entity::belongs_to(super::user::Entity)
                .from(Column::UserId)
                .to(super::user::Column::Id)
                .into(),
            Self::Project => Entity::belongs_to(super::project::Entity)
                .from(Column::ProjectId)
                .to(super::project::Column::Id)
                .into(),
            Self::AuthToken => Entity::belongs_to(super::authtoken::Entity)
                .from(Column::UserId)
                .to(super::authtoken::Column::UserId)
                .into(),
        }
    }
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::authtoken::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AuthToken.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
