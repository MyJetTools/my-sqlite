use async_sqlite::rusqlite::ToSql;

#[derive(Debug)]
pub enum NonStringValue {
    SmallInt(i16),
    Integer(i32),
    BigInt(i64),
    Float(f32),
    Double(f64),
}

impl NonStringValue {
    pub fn to_sql(&self) -> &dyn ToSql {
        match self {
            NonStringValue::SmallInt(value) => value,
            NonStringValue::Integer(value) => value,
            NonStringValue::BigInt(value) => value,
            NonStringValue::Float(value) => value,
            NonStringValue::Double(value) => value,
        }
    }
}

#[derive(Debug)]
pub enum SqlString {
    AsString(String),
    AsStr(&'static str),
    NonStrValue(NonStringValue),
}

impl SqlString {
    pub fn from_str(src: &str) -> Self {
        Self::AsString(src.to_string())
    }

    pub fn from_static_str(src: &'static str) -> Self {
        Self::AsStr(src)
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            SqlString::AsString(value) => Some(value.as_str()),
            SqlString::AsStr(value) => Some(*value),
            _ => None,
        }
    }

    pub fn to_sql(&self) -> &dyn ToSql {
        match self {
            SqlString::AsString(value) => value,
            SqlString::AsStr(value) => value,
            SqlString::NonStrValue(value) => value.to_sql(),
        }
    }
}

impl Into<SqlString> for String {
    fn into(self) -> SqlString {
        SqlString::AsString(self)
    }
}

impl<'s> Into<SqlString> for &'s str {
    fn into(self) -> SqlString {
        SqlString::from_str(self)
    }
}

impl<'s> Into<SqlString> for &'s String {
    fn into(self) -> SqlString {
        SqlString::from_str(self)
    }
}
