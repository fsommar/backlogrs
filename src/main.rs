extern crate iron;
extern crate "rustc-serialize" as rustc_serialize;

use rustc_serialize::{json, Encodable};
use iron::prelude::*;
use iron::status;
use iron::headers;

pub struct Json<T: Encodable>(T);

#[derive(RustcEncodable, RustcDecodable)]
struct Person {
    name: String,
    age: i8
}

fn main() {
    Iron::new(|_: &mut Request| {

        // vim can't handle square brackets on
        // multiple lines apparently, so curly braces
        // are used instead.
        let res = vec!{
            Person {
                name: "fred".to_string(),
                age: 22
            },
            Person {
                name: "dan".to_string(),
                age: 21
            }};

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
