#![feature(core)]
extern crate iron;
extern crate "error" as err;
extern crate "rustc-serialize" as rustc_serialize;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate plugin;
extern crate typemap;

use ::std::iter::FromIterator;
use std::sync::Arc;
use std::default::Default;
use std::error::Error;
use std::fmt::{self, Debug};
use rustc_serialize::{json, Encodable};
use r2d2_postgres::PostgresConnectionManager;
use postgres::SslMode;
use plugin::Extensible;
use iron::prelude::*;
use iron::{
    headers,
    Handler,
};

// Reexport AroundMiddleware for DbConnection so that
// the user doesn't separately need to import it by themselves.
pub use iron::AroundMiddleware;
pub use iron::BeforeMiddleware;
pub use postgres::Row;

pub mod models;

/// Adds an extension method that works like the normal `collect` on
/// iterators but for postgres query results instead. `FromSqlRow` needs
/// to be implemented on the item but that is it.
///
/// Example:
/// ```rust
/// let stmt = db.prepare("SELECT * FROM Person");
/// let res = try!(stmt.query(&[])).collect_sql::<Vec<Person>>();
/// ```
pub trait CollectSql<T> {
    fn collect_sql<R>(self) -> R
        where R: FromIterator<T>;
}

impl<'stmt,T: FromSqlRow> CollectSql<T> for postgres::Rows<'stmt> {
    fn collect_sql<R: FromIterator<T>>(self) -> R {
        self.map(|x| FromSqlRow::from_sql_row(&x)).collect()
    }
}

/// Implement this trait for database models in order for them to be
/// collectable from a postgres query.
///
/// This is a helper trait for `CollectSql` which adds the extension
/// method `collect_sql` to the `Rows` gained from database queries
/// in postgres.
pub trait FromSqlRow {
    fn from_sql_row<'stmt>(row: &postgres::Row<'stmt>) -> Self;
}


#[derive(Debug)]
pub struct LibError;

impl fmt::Display for LibError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            Debug::fmt(self, f)
        }
}

impl Error for LibError {
    fn description(&self) -> &'static str {
        "ApiError"
    }
}

pub struct Api;

impl BeforeMiddleware for Api {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        if req.url.path[0] != "api" {
            Err(IronError::new(LibError, iron::status::NotFound))
        } else {
            // Remove api prefix and continue
            req.url.path[0].clear();
            Ok(())
        }
    }
}

/// A simple wrapper struct for marking a struct as a JSON response.
pub struct Json<T: Encodable>(pub T);

impl<T: Encodable> iron::modifier::Modifier<Response> for Json<T> {
    #[inline]
    fn modify(self, res: &mut Response) {
        let Json(x) = self;
        // Make sure the content type is marked as JSON
        res.headers.set(headers::ContentType("application/json".parse().unwrap()));
        res.set_mut(json::encode(&x).unwrap());
    }
}

/// Provides a convenient extension method for converting an (almost) arbitrary `Result` to
/// an `IronResult`. A constraint for `IronResult` is that the error needs to adhere to
/// `error:Error`. As long as that criteria is met this extension method works.
pub trait OnError<T> {
    /// Converts a `Result` to an `IronResult` by providing the action that should
    /// happen if the `Result` errors out. Anything that goes for a normal response in
    /// an iron `Request` works.
    fn on_err<M: iron::modifier::Modifier<Response>>(self, m: M) -> IronResult<T>;
}

impl<T, E: err::Error> OnError<T> for Result<T, E> {
    fn on_err<M: iron::modifier::Modifier<Response>>(self, m: M) -> IronResult<T> {
        self.map_err(|err| iron::IronError::new(err, m))
    }
}

/// Maintains a database connection pool during requests instead of having
/// to open and close the database for every request.
pub struct DbConnection {
    pool: Arc<r2d2::Pool<PostgresConnectionManager>>
}

impl DbConnection {
    /// Returns a new `DbConnection` with default config and a connection pool
    /// to postgres@/var/run/postgresql not using any SSL.
    pub fn new() -> DbConnection {
        let config = Default::default();
        let manager = PostgresConnectionManager::new(
            // Forward slashes need to be escaped as %2F to be a valid URI.
            // /var/run/postgresql is the default unix socket that when
            // connecting on the same host is automatically accepted
            // even without a password.
            "postgresql://postgres@%2Fvar%2Frun%2Fpostgresql/backlogrs",
            SslMode::None);
        let error_handler = Box::new(r2d2::LoggingErrorHandler);
        let pool = Arc::new(r2d2::Pool::new(config, manager, error_handler).unwrap());
        DbConnection {
            pool: pool
        }
    }
}

impl typemap::Key for DbConnection {
    type Value = Arc<r2d2::Pool<PostgresConnectionManager>>;
}

/// Stores the `DbConnection` that's then used by the `AroundMiddleware`.
struct DbConnectionHandler<H: Handler> {
    conn: DbConnection,
    handler: H
}

/// This `Handler` inserts an `Arc` clone of a `DbConnection` pool before every request.
impl<H: Handler> Handler for DbConnectionHandler<H> {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        req.extensions_mut().insert::<DbConnection>(self.conn.pool.clone());
        self.handler.handle(req)
    }
}

impl AroundMiddleware for DbConnection {
    fn around(self, handler: Box<Handler>) -> Box<Handler> {
        Box::new(DbConnectionHandler {
            conn: self,
            handler: handler
        }) as Box<Handler>
    }
}

/// Provides an extension method for `Request`s to simplify the process of
/// getting the database connection from the `AroundMiddleware` handler.
pub trait GetDb<'a> {
    fn db(&'a self) -> r2d2::PooledConnection<'a, PostgresConnectionManager>;
}

/// Live for at least as long as the borrow on `Request` does.
/// Whether it lives as long as the `Request` itself is not interesting.
impl<'a, 'b: 'a> GetDb<'a> for Request<'b> {
    #[inline]
    fn db(&'a self) -> r2d2::PooledConnection<'a, PostgresConnectionManager> {
        // FIXME: Maybe some form of error handling; e.g. returning an IronResult?
        self.extensions().get::<DbConnection>().unwrap().get().unwrap()
    }
}
