#![feature(core)]
extern crate iron;
extern crate "error" as err;
extern crate "rustc-serialize" as rustc_serialize;
extern crate postgres;

use std::error::{self, Error};
use std::fmt;
use postgres::{Connection, SslMode};
use rustc_serialize::{json, Encodable};
use iron::prelude::*;
use iron::status;
use iron::headers;

pub struct Json<T: Encodable>(T);
pub enum PostgresError {
    Postgres(postgres::ConnectError)
}

impl Error for PostgresError {
    fn description(&self) -> &str {
        match *self {
            PostgresError::Postgres(ref err) => err.description()
        }
    }
}

impl fmt::Display for PostgresError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.description().fmt(f)
    }
}

impl error::FromError<postgres::ConnectError> for Box<PostgresError> {
    fn from_error(err: postgres::ConnectError) -> Box<PostgresError> {
        Box::new(PostgresError::Postgres(err))
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

#[derive(RustcEncodable, RustcDecodable)]
struct Person {
    name: String,
    age: i32
}

fn main() {
    Iron::new(|_: &mut Request| {
        // FIXME: Less unwraps
        // TODO: Don't create a connection for every request?
        // Forward slashes need to be escaped as %2F to be a valid URI
        // FIXME: Remove useless test
        let err: Result<u32, postgres::ConnectError> = Err(postgres::ConnectError::MissingPassword);
        try!(err.on_err("Error!"));
        let conn = try!(Connection::connect(
            "postgresql://postgres@%2Fvar%2Frun%2Fpostgresql",
            &SslMode::None).on_err(status::InternalServerError));

        let stmt = conn.prepare("SELECT * FROM Person").unwrap();
        let res = stmt.query(&[]).unwrap().map(|x| {
            Person {
                name: x.get(0),
                age: x.get(1)
            }
        }).collect::<Vec<Person>>();

        Ok(Response::with((status::Ok, Json(res))))
    }).listen("0.0.0.0:3000").unwrap();
}

impl<T: Encodable> iron::modifier::Modifier<Response> for Json<T> {
    #[inline]
    fn modify(self, res: &mut Response) {
        let Json(x) = self;
        res.headers.set(headers::ContentType("application/json".parse().unwrap()));
        res.set_mut(json::encode(&x).unwrap());
    }
}
