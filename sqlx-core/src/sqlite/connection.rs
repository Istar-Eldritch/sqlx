use core::ptr::{null, null_mut, NonNull};

use std::collections::HashMap;
use std::convert::TryInto;
use std::ffi::CString;

use futures_core::future::BoxFuture;
use futures_util::future;
use libsqlite3_sys::{
    sqlite3, sqlite3_close, sqlite3_extended_result_codes, sqlite3_open_v2, SQLITE_OK,
    SQLITE_OPEN_CREATE, SQLITE_OPEN_NOMUTEX, SQLITE_OPEN_READWRITE, SQLITE_OPEN_SHAREDCACHE,
};

use crate::connection::{Connect, Connection};
use crate::executor::Executor;
use crate::sqlite::statement::SqliteStatement;
use crate::sqlite::Sqlite;
use crate::sqlite::worker::Worker;
use crate::sqlite::SqliteError;
use crate::url::Url;

#[derive(Clone, Copy)]
pub(super) struct SqliteConnectionHandle(NonNull<sqlite3>);

pub struct SqliteConnection {
    pub(super) worker: Worker,
    pub(super) handle: SqliteConnectionHandle,
    // Storage of the most recently prepared, non-persistent statement
    pub(super) statement: Option<SqliteStatement>,
    // Storage of persistent statements
    pub(super) statements: Vec<SqliteStatement>,
    pub(super) statement_by_query: HashMap<String, usize>,
}

// SAFE: A sqlite3 handle is safe to access from multiple threads provided
//       that only one thread access it at a time. Or in other words,
//       the same guarantees that [Sync] requires. This is upheld as long
//       [SQLITE_CONFIG_MULTITHREAD] is enabled and [SQLITE_THREADSAFE] was
//       enabled when sqlite was compiled. We refuse to work if these conditions are
//       not upheld, see [SqliteConnection::establish].
//
// <https://www.sqlite.org/c3ref/threadsafe.html>
// <https://www.sqlite.org/c3ref/c_config_covering_index_scan.html#sqliteconfigmultithread>

#[allow(unsafe_code)]
unsafe impl Send for SqliteConnectionHandle {}

#[allow(unsafe_code)]
unsafe impl Sync for SqliteConnectionHandle {}

async fn establish(url: std::result::Result<Url, url::ParseError>) -> crate::Result<Sqlite, SqliteConnection> {
    let mut worker = Worker::new();

    let url = url?;
    let url = url
        .as_str()
        .trim_start_matches("sqlite:")
        .trim_start_matches("//");

    // By default, we connect to an in-memory database.
    // TODO: Handle the error when there are internal NULs in the database URL
    let filename = CString::new(url).unwrap();

    let handle = worker
        .run(move || -> crate::Result<Sqlite, SqliteConnectionHandle> {
            let mut handle = null_mut();

            // [SQLITE_OPEN_NOMUTEX] will instruct [sqlite3_open_v2] to return an error if it
            // cannot satisfy our wish for a thread-safe, lock-free connection object
            let flags = SQLITE_OPEN_READWRITE
                | SQLITE_OPEN_CREATE
                | SQLITE_OPEN_NOMUTEX
                | SQLITE_OPEN_SHAREDCACHE;

            // <https://www.sqlite.org/c3ref/open.html>
            #[allow(unsafe_code)]
            let status = unsafe { sqlite3_open_v2(filename.as_ptr(), &mut handle, flags, null()) };

            if status != SQLITE_OK {
                return Err(SqliteError::new(status).into());
            }

            // Enable extended result codes
            // https://www.sqlite.org/c3ref/extended_result_codes.html
            #[allow(unsafe_code)]
            unsafe {
                sqlite3_extended_result_codes(handle, 1);
            }

            Ok(SqliteConnectionHandle(NonNull::new(handle).unwrap()))
        })
        .await?;

    Ok(SqliteConnection {
        worker,
        handle,
        statement: None,
        statements: Vec::with_capacity(10),
        statement_by_query: HashMap::with_capacity(10),
    })
}

impl SqliteConnection {
    pub(super) fn handle(&self) -> *mut sqlite3 {
        self.handle.0.as_ptr()
    }
}

impl Connect for SqliteConnection {
    fn connect<T>(url: T) -> BoxFuture<'static, crate::Result<Sqlite, SqliteConnection>>
    where
        T: TryInto<Url, Error = url::ParseError>,
        Self: Sized,
    {
        let url = url.try_into();

        Box::pin(async move {
            let mut conn = establish(url).await?;

            // https://www.sqlite.org/wal.html

            // language=SQLite
            conn.execute(
                r#"
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
                "#,
            )
            .await?;

            Ok(conn)
        })
    }
}

impl Connection for SqliteConnection {
    fn close(self) -> BoxFuture<'static, crate::Result<Sqlite, ()>> {
        // All necessary behavior is handled on drop
        Box::pin(future::ok(()))
    }

    fn ping(&mut self) -> BoxFuture<crate::Result<Sqlite, ()>> {
        // For SQLite connections, PING does effectively nothing
        Box::pin(future::ok(()))
    }
}

impl Drop for SqliteConnection {
    fn drop(&mut self) {
        // Drop all statements first
        self.statements.clear();

        // Next close the statement
        // https://sqlite.org/c3ref/close.html
        #[allow(unsafe_code)]
        unsafe {
            let _ = sqlite3_close(self.handle());
        }
    }
}
