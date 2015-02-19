extern crate iron;
extern crate router;
extern crate backlogrs;

use backlogrs::*;
use backlogrs::models::*;
use iron::prelude::*;
use iron::{status, AroundMiddleware};
use router::Router;

fn main() {
    let mut router = Router::new();
    router.get("/persons", get_persons);
    Iron::new(DbConnection::new().around(Box::new(router))).listen("0.0.0.0:3000").unwrap();
}

fn get_persons(req: &mut Request) -> IronResult<Response> {
    let conn = req.db();
    let err_response = status::InternalServerError;

    let stmt = try!(conn.prepare("SELECT * FROM Person").on_err(err_response));
    let res = try!(stmt.query(&[]).on_err(err_response)).map(|x| {
        Person {
            name: x.get(0),
            age: x.get(1)
        }
    }).collect::<Vec<Person>>();

    Ok(Response::with((status::Ok, Json(res))))
}
