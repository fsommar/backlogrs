extern crate iron;
extern crate router;
extern crate "error" as err;
extern crate "rustc-serialize" as rustc_serialize;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate plugin;
extern crate typemap;

use postgres::SslMode;
use rustc_serialize::{json, Encodable};
use router::Router;
use iron::prelude::*;
use iron::{Handler, AroundMiddleware};
use iron::status;
use iron::headers;
use plugin::Extensible;

use std::sync::Arc;
use std::default::Default;
use r2d2_postgres::PostgresConnectionManager;

pub struct Json<T: Encodable>(T);

impl<T: Encodable> iron::modifier::Modifier<Response> for Json<T> {
    #[inline]
    fn modify(self, res: &mut Response) {
        let Json(x) = self;
        res.headers.set(headers::ContentType("application/json".parse().unwrap()));
        res.set_mut(json::encode(&x).unwrap());
    }
}

trait OnError<T> {
    fn on_err<M: iron::modifier::Modifier<Response>>(self, m: M) -> IronResult<T>;
}

impl<T, E: err::Error> OnError<T> for Result<T, E> {
    fn on_err<M: iron::modifier::Modifier<Response>>(self, m: M) -> IronResult<T> {
        self.map_err(|x| iron::IronError::new(x, m))
    }
}

struct DbConnection {
    pool: Arc<r2d2::Pool<PostgresConnectionManager>>
}

impl DbConnection {
    fn new() -> DbConnection {
        let config = Default::default();
        let manager = PostgresConnectionManager::new(
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

trait GetDb<'a> {
    fn db(&'a self) -> r2d2::PooledConnection<'a, PostgresConnectionManager>;
}

/// Live for at least as long as the borrow on Request does.
/// Whether it lives as long as the Request itself is not interesting.
impl<'a, 'b: 'a> GetDb<'a> for Request<'b> {
    #[inline(always)]
    fn db(&'a self) -> r2d2::PooledConnection<'a, PostgresConnectionManager> {
        self.extensions().get::<DbConnection>().unwrap().get().unwrap()
    }
}

#[derive(RustcEncodable, RustcDecodable)]
struct Person {
    name: String,
    age: i32
}

fn main() {
    let mut router = Router::new();
    router.get("/persons", get_persons);
    Iron::new(DbConnection::new().around(Box::new(router))).listen("0.0.0.0:3000").unwrap();
}

fn get_persons(req: &mut Request) -> IronResult<Response> {
    let conn = req.db();
    let err_response = status::InternalServerError;

    // Forward slashes need to be escaped as %2F to be a valid URI
    let stmt = try!(conn.prepare("SELECT * FROM Person").on_err(err_response));
    let res = try!(stmt.query(&[]).on_err(err_response)).map(|x| {
        Person {
            name: x.get(0),
            age: x.get(1)
        }
    }).collect::<Vec<Person>>();

    Ok(Response::with((status::Ok, Json(res))))
}
