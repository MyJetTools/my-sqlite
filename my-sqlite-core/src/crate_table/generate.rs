use crate::{table_schema::TableSchemaProvider, ColumnName};

pub fn generate_sql_request<T: TableSchemaProvider>(table_name: &str) -> String {
    let primary_key = T::get_primary_key_columns();

    let primary_key_column_amount = if let Some(primary_key) = &primary_key {
        primary_key.len()
    } else {
        0
    };

    let mut result = String::new();

    result.push_str("CREATE TABLE ");

    result.push_str(table_name);

    result.push_str(" (");

    let mut no = 0;

    for column in T::get_columns() {
        if no > 0 {
            result.push_str(",");
        }

        result.push_str(column.name.name.as_str());
        result.push_str(" ");
        result.push_str(column.sql_type.to_db_type());

        if primary_key_column_amount == 1 {
            if let Some(primary_key) = &primary_key {
                for pk in primary_key {
                    if column.name.get_name() == pk.name.as_str() {
                        result.push_str(" PRIMARY KEY");
                        break;
                    }
                }
            }
        }

        no += 1;
    }

    if primary_key_column_amount > 1 {
        result.push_str(",\n");
        result.push_str("  PRIMARY KEY (");
        generate_primary_key_columns(&mut result, primary_key.as_ref().unwrap());

        result.push_str(")");
    }

    result.push_str(")");

    #[cfg(test)]
    println!("Sql: {}", result);

    result
}

fn generate_primary_key_columns(result: &mut String, columns: &[ColumnName]) {
    let mut no = 0;

    for column in columns {
        if no > 0 {
            result.push_str(",");
        }

        result.push_str(column.get_name());

        no += 1;
    }
}
