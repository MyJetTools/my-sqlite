use std::sync::Arc;

use async_sqlite::{ClientBuilder, JournalMode};
use rust_extensions::StrOrString;

use crate::{table_schema::TableSchemaProvider, SqlLiteConnection, SqlLiteError};

pub struct SqlLiteConnectionBuilder {
    path: StrOrString<'static>,
    create_table_sql: Vec<String>,

    debug: bool,
}

impl SqlLiteConnectionBuilder {
    pub fn new(path: impl Into<StrOrString<'static>>) -> Self {
        Self {
            path: path.into(),
            create_table_sql: Vec::with_capacity(4),
            debug: false,
        }
    }

    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn create_table_if_no_exists<T: TableSchemaProvider>(mut self, table_name: &str) -> Self {
        self.create_table_sql
            .push(crate::crate_table::generate_sql_request::<T>(table_name));

        if let Some(indexes) = T::get_indexes() {
            for (name, index_schema) in indexes {
                self.create_table_sql
                    .push(crate::crate_table::generate_create_index_sql(
                        table_name,
                        name.as_str(),
                        index_schema,
                    ));
            }
        }

        self
    }

    fn is_debug(&self) -> bool {
        self.debug || std::env::var("DEBUG").is_ok()
    }

    pub async fn build(self) -> Result<SqlLiteConnection, SqlLiteError> {
        let debug = self.is_debug();
        let client = ClientBuilder::new()
            .path(self.path.as_str())
            .journal_mode(JournalMode::Off)
            .open()
            .await
            .unwrap();

        let result = SqlLiteConnection::new(client, debug).await;

        for create_table_sql in self.create_table_sql {
            let create_table_sql = Arc::new(create_table_sql);

            let create_table_sql_spawned = create_table_sql.clone();

            let result = result
                .client
                .conn(move |connection| connection.execute(create_table_sql_spawned.as_str(), []))
                .await;

            if let Err(err) = &result {
                if debug {
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
