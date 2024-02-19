use std::collections::BTreeMap;

use my_sqlite::macros::*;
use rust_extensions::date_time::DateTimeAsMicroseconds;

#[derive(SelectDbEntity, InsertDbEntity, UpdateDbEntity, TableSchema)]
struct TestEntity {
    #[primary_key]
    #[generate_where_model(name:"WhereByIdModel")]
    pub id: i32,
    #[sql_type("timestamp")]
    pub moment: DateTimeAsMicroseconds,
    #[sql_type("jsonb")]
    pub json_field: BTreeMap<String, String>,
}

#[cfg(test)]
mod tests {
    use my_sqlite::SqlLiteConnectionBuilder;
    use rust_extensions::date_time::DateTimeAsMicroseconds;

    use super::*;

    #[tokio::test]
    async fn test_generate_and_select() {
        const TABLE_NAME: &str = "test_table";
        let connection = SqlLiteConnectionBuilder::new(":memory:")
            .create_table_if_no_exists::<TestEntity>(TABLE_NAME)
            .build()
            .await
            .unwrap();

        let mut json_field = BTreeMap::new();
        json_field.insert("key1".to_string(), "value1".to_string());

        let src_entity = TestEntity {
            id: 2,
            moment: DateTimeAsMicroseconds::from_str("2021-01-02T03:04:05.123456").unwrap(),
            json_field,
        };

        connection
            .insert_db_entity(&src_entity, TABLE_NAME)
            .await
            .unwrap();

        let where_model = WhereByIdModel { id: 2 };

        let result: Option<TestEntity> = connection
            .query_single_row(TABLE_NAME, Some(&where_model))
            .await
            .unwrap();

        let result = result.unwrap();

        assert_eq!(result.id, src_entity.id);
        assert_eq!(
            result.moment.unix_microseconds,
            src_entity.moment.unix_microseconds
        );

        assert_eq!(result.json_field, src_entity.json_field);
    }
}
