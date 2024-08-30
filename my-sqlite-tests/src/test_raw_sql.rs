use my_sqlite::macros::where_raw_model;

#[where_raw_model("moment >= ${from_date} AND moment <= ${to_date} AND (message LIKE '%${phrase}%' OR context LIKE '%${phrase}%')")]
pub struct WhereScanModel<'s> {
    pub from_date: i64,
    pub to_date: i64,
    pub phrase: &'s str,
    pub limit: usize,
}

#[test]
fn test_complicated_where_sql() {
    use my_sqlite::{sql::SqlValues, sql_where::SqlWhereModel};

    let where_model = WhereScanModel {
        from_date: 0,
        to_date: 1,
        phrase: "hello",
        limit: 10,
    };

    let mut params = SqlValues::new();
    let mut sql = String::new();
    where_model.fill_where_component(&mut sql, &mut params);

    println!("sql: {}", sql);
}
