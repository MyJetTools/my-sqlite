use crate::{sql_insert::SqlInsertModel, sql_update::SqlUpdateModel};

use super::{SqlData, UsedColumns};

pub fn build_insert_or_update_sql<'s, TSqlInsertModel: SqlInsertModel + SqlUpdateModel>(
    model: &TSqlInsertModel,
    table_name: &str,
) -> SqlData {
    let mut columns = UsedColumns::new_as_active();
    let sql_data = super::build_insert_sql(
        super::InsertType::OrReplace,
        model,
        table_name,
        &mut columns,
    );

    // update_conflict_type.generate_sql(&mut sql_data.sql);

    //sql_data.sql.push_str(" DO UPDATE SET ");

    //    TSqlInsertModel::fill_upsert_sql_part(&mut sql_data.sql, &columns.into());

    sql_data
}
