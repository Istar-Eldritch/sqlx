use std::fmt::{self, Display};
use std::sync::Arc;

use crate::error::DatabaseError;
use crate::postgres::protocol::Response;

#[derive(Debug)]
pub struct PgError {
    pub(super) response: Response,
    pub(super) query: Option<Arc<str>>,
}

impl DatabaseError for PgError {
    fn message(&self) -> &str {
        self.response.message()
    }

    fn code(&self) -> Option<&str> {
        Some(&self.response.message())
    }

    fn details(&self) -> Option<&str> {
        self.field(b'D')
    }

    fn hint(&self) -> Option<&str> {
        self.field(b'H')
    }

    fn table_name(&self) -> Option<&str> {
        self.field(b't')
    }

    fn column_name(&self) -> Option<&str> {
        self.field(b'c')
    }

    fn constraint_name(&self) -> Option<&str> {
        self.field(b'n')
    }
}

impl PgError {
    /// Get a field from the error
    pub fn field(&self, tag: u8) -> Option<&str> {
        self.response.field(tag)
    }
}

impl Display for PgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(line) = self.response.field(b'L')
        f.pad(self.message())
    }
}

impl crate::Error {
    pub(crate) fn pg_err_attach_query(self, get_query: impl FnOnce() -> Arc<str>) -> Self {

    }
}