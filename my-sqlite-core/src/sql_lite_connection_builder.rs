use std::sync::Arc;

use async_sqlite::{ClientBuilder, JournalMode};
use rust_extensions::StrOrString;

use crate::{table_schema::TableSchemaProvider, SqlLiteConnection, SqlLiteError};

pub struct SqlLiteConnectionBuilder {
    path: StrOrString<'static>,
    create_table_sql: Vec<String>,
}

impl SqlLiteConnectionBuilder {
    pub fn new(path: impl Into<StrOrString<'static>>) -> Self {
        Self {
            path: path.into(),
            create_table_sql: Vec::with_capacity(4),
        }
    }

    pub fn create_table_if_no_exists<T: TableSchemaProvider>(mut self, table_name: &str) -> Self {
        self.create_table_sql
            .push(crate::crate_table::generate_sql_request::<T>(table_name));
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

        for create_table_sql in self.create_table_sql {
            let create_table_sql = Arc::new(create_table_sql);

            let create_table_sql_spawned = create_table_sql.clone();

            let result = result
                .client
                .conn(move |connection| connection.execute(create_table_sql_spawned.as_str(), []))
                .await;

            if let Err(err) = &result {
                if std::env::var("DEBUG").is_ok() {
                    println!("Sql:{}", create_table_sql.as_str());
                }

                let mut skip_error = false;

                match err {
                    async_sqlite::Error::Rusqlite(err) => match err {
                        async_sqlite::rusqlite::Error::SqlInputError {
                            error,
                            msg: _,
                            sql: _,
                            offset: _,
                        } => match error.code {
                            async_sqlite::rusqlite::ErrorCode::Unknown => {
                                if error.extended_code == 1 {
                                    //Imperially this error means that the table already exists.
                                    skip_error = true;
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    },
                    _ => {}
                }

                if !skip_error {
                    panic!("{:?}", err);
                }
            }
        }

        Ok(result)
    }
}
