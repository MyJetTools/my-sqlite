use async_sqlite::rusqlite::types::FromSql;

use crate::{
    sql::SelectBuilder,
    sql_select::{DbColumnName, FromDbRow, SelectValueProvider},
    GroupByFieldType, SqlValueMetadata,
};

#[derive(Debug)]
pub struct GroupByAvg<T: std::fmt::Debug + Send + Sync + 'static>(T);

impl<T: std::fmt::Debug + Copy + FromSql + Send + Sync + 'static> GroupByAvg<T> {
    pub fn get_value(&self) -> T {
        self.0
    }
}

impl<T: std::fmt::Debug + GroupByFieldType + Send + Sync + 'static> SelectValueProvider
    for GroupByAvg<T>
{
    fn fill_select_part(
        sql: &mut SelectBuilder,
        field_name: DbColumnName,
        metadata: &Option<SqlValueMetadata>,
    ) {
        let sql_type = if let Some(metadata) = metadata {
            if let Some(sql_type) = metadata.sql_type {
                sql_type
            } else {
                T::DB_SQL_TYPE
            }
        } else {
            T::DB_SQL_TYPE
        };

        sql.push(crate::sql::SelectFieldValue::GroupByField {
            field_name,
            statement: format!("AVG({})::{}", field_name.db_column_name, sql_type).into(),
        });
    }
}

impl<'s, T: std::fmt::Debug + Copy + FromSql + Send + Sync + 'static> FromDbRow<'s, GroupByAvg<T>>
    for GroupByAvg<T>
{
    fn from_db_row(
        row: &'s crate::DbRow,
        column_name: DbColumnName,
        _metadata: &Option<SqlValueMetadata>,
    ) -> GroupByAvg<T> {
        let value: T = row.get(column_name.db_column_name);
        GroupByAvg(value)
    }

    fn from_db_row_opt(
        row: &'s crate::DbRow,
        column_name: DbColumnName,
        _metadata: &Option<SqlValueMetadata>,
    ) -> Option<GroupByAvg<T>> {
        let result: Option<T> = row.get(column_name.db_column_name);
        Some(GroupByAvg(result?))
    }
}
