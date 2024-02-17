use crate::{sql::SelectBuilder, sql_where::SqlWhereModel, DbRow};

use super::BulkSelectBuilder;

pub trait BulkSelectEntity {
    fn get_line_no(&self) -> i32;
}

pub trait SelectEntity {
    fn from(row: &DbRow) -> Self;
    fn fill_select_fields(select_builder: &mut SelectBuilder);

    fn get_select_fields() -> &'static [&'static str];

    fn get_order_by_fields() -> Option<&'static str>;
    fn get_group_by_fields() -> Option<&'static str>;

    fn build_bulk_select<TWhereModel: SqlWhereModel>(
        table_name: &'static str,
        where_models: Vec<TWhereModel>,
    ) -> BulkSelectBuilder<TWhereModel> {
        BulkSelectBuilder::new(table_name, where_models)
    }

    fn into_select_builder<TSelectEntity: SelectEntity>() -> SelectBuilder {
        SelectBuilder::from_select_model::<TSelectEntity>()
    }
}
