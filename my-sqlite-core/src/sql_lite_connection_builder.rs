use std::sync::Arc;

use async_sqlite::{ClientBuilder, JournalMode};
use rust_extensions::StrOrString;

use crate::{table_schema::TableSchemaProvider, SqlLiteConnection, SqlLiteError};

pub struct SqlLiteConnectionBuilder {
    path: StrOrString<'static>,
    create_table_sql: Option<String>,
}

impl SqlLiteConnectionBuilder {
    pub fn new(path: impl Into<StrOrString<'static>>) -> Self {
        Self {
            path: path.into(),
            create_table_sql: None,
        }
    }

    pub fn create_table_if_no_exists<T: TableSchemaProvider>(mut self, table_name: &str) -> Self {
        self.create_table_sql = Some(crate::crate_table::generate_sql_request::<T>(table_name));
        self
    }

    pub async fn build(self) -> Result<SqlLiteConnection, SqlLiteError> {
        let client = ClientBuilder::new()
            .path(self.path.as_str())
            .journal_mode(JournalMode::Off)
            .open()
            .await
            .unwrap();

        let result = SqlLiteConnection::new(client).await;

        if let Some(create_table_sql) = self.create_table_sql {
            let create_table_sql = Arc::new(create_table_sql);

            let create_table_sql_spawned = create_table_sql.clone();

            let result = result
                .client
                .conn(move |connection| connection.execute(create_table_sql_spawned.as_str(), []))
                .await;

            if let Err(err) = result {
                println!("Sql:{}", create_table_sql.as_str());

                panic!("{:?}", err);
            }
        }

        Ok(result)
    }
}
