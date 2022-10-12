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
    pub handler_type: HandlerParams,
    #[sea_orm(column_type = "Text")]
    pub index: String,
    pub url: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod test {
    use mars_config::ProxyParams;
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
            id: sea_orm::ActiveValue::Set(11),
            project_id: sea_orm::ActiveValue::Set(1),
            method: sea_orm::ActiveValue::Set(Method(mars_config::Method::GET)),
            query_params: sea_orm::ActiveValue::Set(QueryParams(vec![QueryParam {
                key: "haha".to_owned(),
                value: "haha".to_owned(),
                action: Action::Add,
            }])),
            headers: sea_orm::ActiveValue::Set(Headers(vec![Header {
                key: "haha".to_owned(),
                value: "haha".to_owned(),
                action: Action::Add,
            }])),
            handler_type: sea_orm::ActiveValue::Set(HandlerParams(ProxyParams {
                params: json!({
                    "password": "password",
                    "username": "prasanth"
                }),
                handler_type: "no_auth".to_string(),
            })),
            index: sea_orm::ActiveValue::Set("test2".to_owned()),
            url: sea_orm::ActiveValue::Set("https://httpbin.org/".to_owned()),
        };
        let res = Entity::insert(pear).exec(&db).await.unwrap();

        println!();
        println!("Inserted: last_insert_id = {}\n", res.last_insert_id);
    }
}
