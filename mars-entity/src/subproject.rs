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
    pub query_params: QueryParams,
    pub headers: Headers,
    pub auth: Auth,
    pub params: GeneralParams,
    pub index: String,
    pub url: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod test {
    use sea_orm::{sea_query::TableCreateStatement, ConnectionTrait, Database, Schema};
    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn test_subproject_query_n_insert() {
        let path = format!(
            "sqlite://{}/db.sqlite?mode=rwc",
            std::env::current_dir()
                .expect("unable to figure out directory")
                .to_str()
                .unwrap()
        );
        println!("path is {}", path);

        let db = Database::connect(path).await.unwrap();

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
            auth: sea_orm::ActiveValue::Set(Auth(mars_config::MarsAuth::new(
                json!([{
                    "key": "Authorization",
                    "value": "Bearer hai"
                }]),
                mars_config::AuthType::HeaderAuth,
            ))),
            params: sea_orm::ActiveValue::Set(GeneralParams(mars_config::GeneralParams::new(
                json!({}),
            ))),
            index: sea_orm::ActiveValue::Set("userinput".to_owned()),
            url: sea_orm::ActiveValue::Set("https://httpbin.org/".to_owned()),
        };
        let res = Entity::insert(pear).exec(&db).await.unwrap();

        let result = Entity::find()
            .filter(Column::Id.eq(res.last_insert_id))
            .one(&db)
            .await
            .unwrap();

        println!("Result: {:?}", result);
        println!("Inserted: last_insert_id = {}\n", res.last_insert_id);
    }
}
