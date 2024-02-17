use crate::table_schema::TableSchemaProvider;

pub fn generate_sql_request<T: TableSchemaProvider>(table_name: &str) -> String {
    let mut result = String::new();

    result.push_str("CREATE TABLE ");

    result.push_str(table_name);

    result.push_str(" (");

    let mut no = 0;

    let primary_key = T::get_primary_key_columns();

    for column in T::get_columns() {
        if no > 0 {
            result.push_str(",");
        }

        result.push_str(column.name.name.as_str());
        result.push_str(" ");
        result.push_str(column.sql_type.to_db_type());

        if let Some(primary_key) = &primary_key {
            for pk in primary_key {
                if pk.name.as_str() == pk.name.as_str() {
                    result.push_str(" PRIMARY KEY");
                    break;
                }
            }
        }

        no += 1;
    }

    result.push_str(")");

    #[cfg(test)]
    println!("Sql: {}", result);

    result
}
