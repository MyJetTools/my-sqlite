#[derive(Debug)]
pub enum SqlLiteError {
    SqlLiteError(async_sqlite::Error),
    RusSqliteError(async_sqlite::rusqlite::Error),
}

impl From<async_sqlite::Error> for SqlLiteError {
    fn from(value: async_sqlite::Error) -> Self {
        Self::SqlLiteError(value)
    }
}

impl From<async_sqlite::rusqlite::Error> for SqlLiteError {
    fn from(value: async_sqlite::rusqlite::Error) -> Self {
        Self::RusSqliteError(value)
    }
}
