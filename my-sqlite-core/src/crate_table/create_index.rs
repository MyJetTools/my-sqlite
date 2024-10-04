use crate::table_schema::{IndexField, IndexSchema};

pub fn generate_create_index_sql(table_name: &str, index_name: &str, index: IndexSchema) -> String {
    let mut result = String::new();

    if index.is_unique {
        result.push_str("CREATE UNIQUE INDEX IF NOT EXISTS ");
    } else {
        result.push_str("CREATE INDEX IF NOT EXISTS ");
    }

    result.push_str(index_name);

    result.push_str(" ON ");

    result.push_str(table_name);

    result.push_str("(");
    generate_fields(&mut result, &index.fields);
    result.push_str(")");

    return result;
}

fn generate_fields(result: &mut String, fields: &Vec<IndexField>) {
    let mut i = 0;
    for field in fields.iter() {
        if i > 0 {
            result.push(',');
        }
        field.name.push_name(result);

        i += 1;
    }
}
