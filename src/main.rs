extern crate iron;
extern crate router;
extern crate "error" as err;
extern crate "rustc-serialize" as rustc_serialize;
extern crate postgres;
extern crate plugin;
extern crate typemap;

use postgres::{Connection, SslMode};
use rustc_serialize::{json, Encodable};
use router::Router;
use iron::prelude::*;
use iron::{Handler, AroundMiddleware};
use iron::status;
use iron::headers;
use plugin::Extensible;

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

struct DbConnection;
impl typemap::Key for DbConnection { type Value = postgres::Connection; }

struct DbConnectionHandler<H: Handler> {
    handler: H
}

// TODO: Don't create a connection for every request
// r2d2 and r2d2_postgres could be used for creating a db pool
impl<H: Handler> Handler for DbConnectionHandler<H> {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let conn = try!(Connection::connect(
                "postgresql://postgres@%2Fvar%2Frun%2Fpostgresql",
                &SslMode::None).on_err(status::InternalServerError));
        req.extensions_mut().insert::<DbConnection>(conn);
        let res = self.handler.handle(req);
        res
    }
}

impl AroundMiddleware for DbConnection {
    fn around(self, handler: Box<Handler>) -> Box<Handler> {
        Box::new(DbConnectionHandler {
            handler: handler
        }) as Box<Handler>
    }
}

trait GetDb {
    fn db(&self) -> &postgres::Connection;
}

impl<'a> GetDb for Request<'a> {
    fn db(&self) -> &postgres::Connection {
        self.extensions().find::<DbConnection>().unwrap()
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
    Iron::new(DbConnection.around(Box::new(router))).listen("0.0.0.0:3000").unwrap();
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
