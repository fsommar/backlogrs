extern crate iron;
extern crate "error" as err;
extern crate "rustc-serialize" as rustc_serialize;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate plugin;
extern crate typemap;

use std::sync::Arc;
use std::default::Default;
use rustc_serialize::{json, Encodable};
use r2d2_postgres::PostgresConnectionManager;
use postgres::SslMode;
use plugin::Extensible;
use iron::prelude::*;
use iron::{
    headers,
    Handler,
    AroundMiddleware
};

pub mod models;

pub struct Json<T: Encodable>(pub T);

impl<T: Encodable> iron::modifier::Modifier<Response> for Json<T> {
    #[inline]
    fn modify(self, res: &mut Response) {
        let Json(x) = self;
        res.headers.set(headers::ContentType("application/json".parse().unwrap()));
        res.set_mut(json::encode(&x).unwrap());
    }
}

pub trait OnError<T> {
    fn on_err<M: iron::modifier::Modifier<Response>>(self, m: M) -> IronResult<T>;
}

impl<T, E: err::Error> OnError<T> for Result<T, E> {
    fn on_err<M: iron::modifier::Modifier<Response>>(self, m: M) -> IronResult<T> {
        self.map_err(|x| iron::IronError::new(x, m))
    }
}

pub struct DbConnection {
    pool: Arc<r2d2::Pool<PostgresConnectionManager>>
}

impl DbConnection {
    pub fn new() -> DbConnection {
        let config = Default::default();
        let manager = PostgresConnectionManager::new(
            // Forward slashes need to be escaped as %2F to be a valid URI
            "postgresql://postgres@%2Fvar%2Frun%2Fpostgresql",
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

struct DbConnectionHandler<H: Handler> {
    conn: DbConnection,
    handler: H
}

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

pub trait GetDb<'a> {
    fn db(&'a self) -> r2d2::PooledConnection<'a, PostgresConnectionManager>;
}

/// Live for at least as long as the borrow on Request does.
/// Whether it lives as long as the Request itself is not interesting.
impl<'a, 'b: 'a> GetDb<'a> for Request<'b> {
    #[inline]
    fn db(&'a self) -> r2d2::PooledConnection<'a, PostgresConnectionManager> {
        self.extensions().get::<DbConnection>().unwrap().get().unwrap()
    }
}
