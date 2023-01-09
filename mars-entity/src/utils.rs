#[macro_export]
macro_rules! impl_mandatory_types {
    ($name:ident) => {
        impl From<$name> for sea_orm::Value {
            fn from(value: $name) -> Self {
                sea_orm::Value::String(Some(Box::new(serde_json::to_string(&value).unwrap())))
            }
        }

        impl ValueType for $name {
            fn try_from(v: Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
                match v {
                    Value::String(Some(x)) => {
                        Ok(serde_json::from_str(&x)
                            .map_err(|_| sea_orm::sea_query::ValueTypeErr)?)
                    }
                    _ => Err(sea_orm::sea_query::ValueTypeErr),
                }
            }

            fn type_name() -> String {
                stringify!(QueryParams).to_string()
            }

            fn column_type() -> sea_orm::sea_query::ColumnType {
                sea_orm::sea_query::ColumnType::String(None)
            }
        }

        impl sea_orm::TryGetableFromJson for $name {}
    };
}
