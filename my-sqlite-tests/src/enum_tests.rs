use my_sqlite::macros::DbEnumAsString;

#[derive(DbEnumAsString, Debug, Clone)]
pub enum LogLevelDto {
    Info,
    Warning,
    Error,
    FatalError,
    Debug,
}
