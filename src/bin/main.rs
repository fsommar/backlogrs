extern crate iron;
extern crate router;
extern crate backlogrs;

use backlogrs::*;
use backlogrs::models::*;
use iron::prelude::*;
use iron::{status};
use router::Router;

fn main() {
    let mut router = Router::new();
    router.get("/persons", get_persons);

    let mut chain = Chain::new(router);
    chain.link_before(Api);
    chain.around(DbConnection::new());

    Iron::new(chain).listen("0.0.0.0:3000").unwrap();
}

fn get_persons(req: &mut Request) -> IronResult<Response> {
    let db = req.db();
    let e = status::InternalServerError;

    let stmt = try!(db.prepare("SELECT * FROM Person").on_err(e));
    let res = try!(stmt.query(&[]).on_err(e)).map(|x| {
        Person {
            name: x.get(0),
            age: x.get(1)
        }
    }).collect::<Vec<Person>>();

    Ok(Response::with((status::Ok, Json(res))))
}
