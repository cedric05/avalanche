use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

pub use super::query::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "subproject")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub project_id: i32,
    pub method: Method,
    #[sea_orm(column_type = "Text")]
    pub query_params: QueryParams,
    #[sea_orm(column_type = "Text")]
    pub headers: Headers,
    #[sea_orm(column_type = "Text")]
    pub auth: Auth,
    #[sea_orm(column_type = "Text")]
    pub params: GeneralParams,
    #[sea_orm(column_type = "Text")]
    pub index: String,
    pub url: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod test {
    use sea_orm::{sea_query::TableCreateStatement, ConnectionTrait, Database, Schema, Set};
    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn haha() {
        let db = Database::connect(
            "sqlite:///home/neptune/projects/personal/cedric05/mars-rover/db.sqlite",
        )
        .await
        .unwrap();

        let builder = db.get_database_backend();
        let schema = Schema::new(builder);
        // Derive from Entity
        let stmt: TableCreateStatement = schema.create_table_from_entity(Entity);

        let result = db.execute(db.get_database_backend().build(&stmt)).await;
        println!("{:?}\n {result:?}", db);

        let pear = ActiveModel {
            project_id: sea_orm::ActiveValue::Set(1),
            // auto increment
            id: sea_orm::ActiveValue::NotSet,
            method: sea_orm::ActiveValue::Set(Method(mars_config::Method::GET)),
            query_params: sea_orm::ActiveValue::Set(QueryParams(vec![QueryParam {
                key: "querykey".to_owned(),
                value: "queryvalue".to_owned(),
                action: Action::Add,
            }])),
            headers: sea_orm::ActiveValue::Set(Headers(vec![Header {
                key: "headerkey".to_owned(),
                value: "headervalue".to_owned(),
                action: Action::Add,
            }])),
            auth: sea_orm::ActiveValue::Set(Auth(mars_config::MarsAuth {
                params: json!({
                    "password": "password",
                    "username": "postman"
                }),
                auth_type: mars_config::AuthType::DigestAuth,
            })),
            params: sea_orm::ActiveValue::Set(GeneralParams(mars_config::GeneralParams(json!({})))),
            index: sea_orm::ActiveValue::Set("digest2".to_owned()),
            url: sea_orm::ActiveValue::Set("https://postman-echo.com/".to_owned()),
        };
        let res = Entity::insert(pear).exec(&db).await.unwrap();

        println!();
        println!("Inserted: last_insert_id = {}\n", res.last_insert_id);
    }
}
