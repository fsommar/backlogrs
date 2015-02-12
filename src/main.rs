extern crate iron;
extern crate "rustc-serialize" as rustc_serialize;
extern crate postgres;

use postgres::{Connection, SslMode};

use rustc_serialize::{json, Encodable};
use iron::prelude::*;
use iron::status;
use iron::headers;

pub struct Json<T: Encodable>(T);

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
        let conn = Connection::connect(
            "postgresql://postgres@%2Fvar%2Frun%2Fpostgresql",
            &SslMode::None).unwrap();

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
