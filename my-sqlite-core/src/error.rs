#[derive(Debug)]
pub enum SqlLiteError {
    SqlLiteError(async_sqlite::Error),
}

impl From<async_sqlite::Error> for SqlLiteError {
    fn from(value: async_sqlite::Error) -> Self {
        Self::SqlLiteError(value)
    }
}
