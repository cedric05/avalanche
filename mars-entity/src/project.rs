use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "project")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub index: String,
    pub needs_auth: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod test {
    use super::*;
    use sea_orm::{sea_query::TableCreateStatement, ConnectionTrait, Database, Schema};

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
            id: sea_orm::ActiveValue::Set(1),
            index: sea_orm::ActiveValue::Set("test".to_owned()),
            needs_auth: sea_orm::ActiveValue::Set(false),
        };
        let res = Entity::insert(pear).exec(&db).await.unwrap();

        println!();
        println!("Inserted: last_insert_id = {}\n", res.last_insert_id);
    }
}
