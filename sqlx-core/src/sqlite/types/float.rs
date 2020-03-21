use crate::decode::Decode;
use crate::encode::Encode;
use crate::sqlite::types::{SqliteType, SqliteTypeAffinity};
use crate::sqlite::{Sqlite, SqliteArgumentValue, SqliteResultValue, SqliteTypeInfo};
use crate::types::Type;

impl Type<Sqlite> for f32 {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo::new(SqliteType::Float, SqliteTypeAffinity::Real)
    }
}

impl Encode<Sqlite> for f32 {
    fn encode(&self, values: &mut Vec<SqliteArgumentValue>) {
        values.push(SqliteArgumentValue::Double((*self).into()));
    }
}

impl<'a> Decode<'a, Sqlite> for f32 {
    fn decode(value: SqliteResultValue<'a>) -> crate::Result<Sqlite, f32> {
        Ok(value.double() as f32)
    }
}

impl Type<Sqlite> for f64 {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo::new(SqliteType::Float, SqliteTypeAffinity::Real)
    }
}

impl Encode<Sqlite> for f64 {
    fn encode(&self, values: &mut Vec<SqliteArgumentValue>) {
        values.push(SqliteArgumentValue::Double((*self).into()));
    }
}

impl<'a> Decode<'a, Sqlite> for f64 {
    fn decode(value: SqliteResultValue<'a>) -> crate::Result<Sqlite, f64> {
        Ok(value.double())
    }
}
