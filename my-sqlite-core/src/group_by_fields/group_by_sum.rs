use async_sqlite::rusqlite::types::FromSql;

use crate::{
    sql::SelectBuilder,
    sql_select::{DbColumnName, FromDbRow, SelectValueProvider},
    GroupByFieldType, SqlValueMetadata,
};

pub struct GroupBySum<T: Send + Sync + 'static>(T);

impl<'s, T: Copy + FromSql + Send + Sync + 'static> GroupBySum<T> {
    pub fn get_value(&self) -> T {
        self.0
    }
}

impl<'s, T: GroupByFieldType + Send + Sync + 'static> SelectValueProvider for GroupBySum<T> {
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
            statement: format!("cast(SUM({}) as {})", field_name.db_column_name, sql_type).into(),
        });
    }
}

impl<'s, T: Copy + FromSql + Send + Sync + 'static> FromDbRow<'s, GroupBySum<T>> for GroupBySum<T> {
    fn from_db_row(
        row: &'s crate::DbRow,
        column_name: DbColumnName,
        _metadata: &Option<SqlValueMetadata>,
    ) -> GroupBySum<T> {
        GroupBySum(row.get(column_name.db_column_name))
    }

    fn from_db_row_opt(
        row: &'s crate::DbRow,
        column_name: DbColumnName,
        _metadata: &Option<SqlValueMetadata>,
    ) -> Option<GroupBySum<T>> {
        let result: Option<T> = row.get(column_name.db_column_name);
        Some(GroupBySum(result?))
    }
}
