pub trait CountResult {
    fn get_postgres_type() -> &'static str;
}

impl CountResult for u64 {
    fn get_postgres_type() -> &'static str {
        "bigint"
    }
}

impl CountResult for i32 {
    fn get_postgres_type() -> &'static str {
        "int"
    }
}

impl CountResult for usize {
    fn get_postgres_type() -> &'static str {
        "bigint"
    }
}

impl CountResult for i16 {
    fn get_postgres_type() -> &'static str {
        "smallint"
    }
}
