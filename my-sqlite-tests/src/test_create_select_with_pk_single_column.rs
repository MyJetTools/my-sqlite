use my_sqlite::macros::*;

#[derive(SelectDbEntity, InsertDbEntity, UpdateDbEntity, TableSchema)]
struct TestEntity {
    #[primary_key]
    #[generate_where_model(name:"WhereByIdModel")]
    pub id: i32,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use my_sqlite::{sql_where::NoneWhereModel, SqlLiteConnectionBuilder};

    use super::*;

    #[tokio::test]
    async fn test_generate_and_select() {
        const TABLE_NAME: &str = "test_table";
        let connection = SqlLiteConnectionBuilder::new(":memory:")
            .create_table_if_no_exists::<TestEntity>(TABLE_NAME)
            .build()
            .await
            .unwrap();

        let src_entity = TestEntity {
            id: 2,
            name: "test".to_string(),
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
        assert_eq!(result.name, src_entity.name);
    }

    #[tokio::test]
    async fn test_generate_and_bulk_insert() {
        const TABLE_NAME: &str = "test_table";
        let connection = SqlLiteConnectionBuilder::new(":memory:")
            .create_table_if_no_exists::<TestEntity>(TABLE_NAME)
            .build()
            .await
            .unwrap();

        let to_insert = vec![
            TestEntity {
                id: 2,
                name: "test".to_string(),
            },
            TestEntity {
                id: 3,
                name: "test2".to_string(),
            },
        ];

        connection
            .bulk_insert_db_entities(&to_insert, TABLE_NAME)
            .await
            .unwrap();

        let result: Vec<TestEntity> = connection
            .query_rows(TABLE_NAME, Some(&NoneWhereModel))
            .await
            .unwrap();

        assert_eq!(2, result.len());
    }

    #[tokio::test]
    async fn test_bulk_insert_or_skip() {
        const TABLE_NAME: &str = "test_table";
        let connection = SqlLiteConnectionBuilder::new(":memory:")
            .create_table_if_no_exists::<TestEntity>(TABLE_NAME)
            .build()
            .await
            .unwrap();

        connection
            .insert_db_entity(
                &TestEntity {
                    id: 2,
                    name: "test_before".to_string(),
                },
                TABLE_NAME,
            )
            .await
            .unwrap();

        let to_insert = [
            TestEntity {
                id: 2,
                name: "test".to_string(),
            },
            TestEntity {
                id: 3,
                name: "test2".to_string(),
            },
        ];

        connection
            .bulk_insert_db_entities_if_not_exists(&to_insert, TABLE_NAME)
            .await
            .unwrap();

        let result: Vec<TestEntity> = connection
            .query_rows(TABLE_NAME, Some(&NoneWhereModel))
            .await
            .unwrap();

        assert_eq!(2, result.len());
    }

    #[tokio::test]
    async fn test_bulk_insert_or_update() {
        const TABLE_NAME: &str = "test_table";
        let connection = SqlLiteConnectionBuilder::new(":memory:")
            .create_table_if_no_exists::<TestEntity>(TABLE_NAME)
            .build()
            .await
            .unwrap();

        connection
            .insert_db_entity(
                &TestEntity {
                    id: 2,
                    name: "test_before".to_string(),
                },
                TABLE_NAME,
            )
            .await
            .unwrap();

        let to_insert = [
            TestEntity {
                id: 2,
                name: "test".to_string(),
            },
            TestEntity {
                id: 3,
                name: "test2".to_string(),
            },
        ];

        connection
            .bulk_insert_or_update(&to_insert, TABLE_NAME)
            .await
            .unwrap();

        let result: Vec<TestEntity> = connection
            .query_rows(TABLE_NAME, Some(&NoneWhereModel))
            .await
            .unwrap();

        assert_eq!(2, result.len());
    }
}
