use async_sqlite::rusqlite::types::FromSql;

use crate::{
    sql::SelectBuilder,
    sql_select::{FromDbRow, SelectValueProvider},
    GroupByFieldType, SqlValueMetadata,
};

pub struct GroupByMin<T: Send + Sync + 'static>(T);

impl<'s, T: Copy + FromSql + Send + Sync + 'static> GroupByMin<T> {
    pub fn get_value(&self) -> T {
        self.0
    }
}

impl<'s, T: GroupByFieldType + Send + Sync + 'static> SelectValueProvider for GroupByMin<T> {
    fn fill_select_part(
        sql: &mut SelectBuilder,
        field_name: &'static str,
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
            statement: format!("cast(MIN({field_name}) as {})", sql_type).into(),
        });
    }
}

impl<'s, T: Copy + FromSql + Send + Sync + 'static> FromDbRow<'s, GroupByMin<T>> for GroupByMin<T> {
    fn from_db_row(
        row: &'s crate::DbRow,
        name: &str,
        _metadata: &Option<SqlValueMetadata>,
    ) -> GroupByMin<T> {
        GroupByMin(row.get(name))
    }

    fn from_db_row_opt(
        row: &'s crate::DbRow,
        name: &str,
        _metadata: &Option<SqlValueMetadata>,
    ) -> Option<GroupByMin<T>> {
        let result: Option<T> = row.get(name);
        Some(GroupByMin(result?))
    }
}
