use std::collections::BTreeMap;

use my_sqlite::macros::*;
use types_reader::rust_extensions::date_time::DateTimeAsMicroseconds;

#[derive(DbEnumAsString, Debug, Clone)]
pub enum LogLevelDto {
    Info,
    Warning,
    Error,
    FatalError,
    Debug,
}

#[derive(TableSchema, InsertDbEntity, SelectDbEntity, Debug)]
pub struct LogItemDto {
    #[primary_key(0)]
    #[sql_type("timestamp")]
    #[order_by_desc]
    pub moment: DateTimeAsMicroseconds,
    #[db_index(id:0, index_name:"id_idx", is_unique:true, order:"ASC")]
    pub id: String,
    pub level: LogLevelDto,
    pub message: String,
    #[sql_type("jsonb")]
    pub context: BTreeMap<String, String>,
}

#[derive(WhereDbModel)]
pub struct WhereModel {
    #[sql_type("timestamp")]
    #[db_column_name("moment")]
    #[operator(">=")]
    pub from_date: DateTimeAsMicroseconds,
    #[sql_type("timestamp")]
    #[ignore_if_none]
    #[db_column_name("moment")]
    #[operator("<=")]
    pub to_date: Option<DateTimeAsMicroseconds>,
    #[ignore_if_none]
    pub level: Option<Vec<LogLevelDto>>,
    #[sql_type("jsonb")]
    #[ignore_if_none]
    pub context: Option<BTreeMap<String, String>>,
    #[limit]
    pub take: usize,
}

#[cfg(test)]
mod tests {
    use my_sqlite::SqlLiteConnectionBuilder;

    use crate::test_log_dto::*;

    #[tokio::test]
    async fn test_save_and_get_record() {
        const TABLE_NAME: &str = "logs";
        let connection = SqlLiteConnectionBuilder::new(":memory:")
            .create_table_if_no_exists::<LogItemDto>(TABLE_NAME)
            .build()
            .await
            .unwrap();

        let now = DateTimeAsMicroseconds::now();

        let mut context = BTreeMap::new();
        context.insert("Test".to_string(), "Test".to_string());

        let dto = LogItemDto {
            moment: now,
            id: "123".to_string(),
            level: LogLevelDto::Info,
            message: "test Message".to_string(),
            context,
        };

        connection.insert_db_entity(&dto, TABLE_NAME).await.unwrap();

        let where_model = WhereModel {
            from_date: now,
            to_date: None,
            level: None,
            context: None,
            take: 10,
        };

        let result: Vec<LogItemDto> = connection
            .query_rows(TABLE_NAME, Some(&where_model))
            .await
            .unwrap();

        assert_eq!(1, result.len());
    }
}
