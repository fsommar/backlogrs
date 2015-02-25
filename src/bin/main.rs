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
    router.get("/user", get_users);
    router.get("/user/:id", get_user_by_id);
    router.get("/user/:id/library", get_library);
    router.get("/user/:id/library/:id", get_entry);
    router.get("/game", get_games);
    router.get("/game/:id", get_game_by_id);
    router.get("/status", get_status);

    let mut chain = Chain::new(router);
    chain.link_before(Api);
    chain.around(DbConnection::new());

    Iron::new(chain).listen("0.0.0.0:3000").unwrap();
    println!("Listening on port 3000...");
}

fn get_status(req: &mut Request) -> IronResult<Response> {
    let db = req.db();
    let e = status::InternalServerError;

    let stmt = try!(db.prepare("SELECT unnest(enum_range(NULL::Status))").on_err(e));
    let res = try!(stmt.query(&[]).on_err(e)).map(|x| {
        x.get(0)
    }).collect::<Vec<Status>>();

    Ok(Response::with((status::Ok, Json(res))))
}

fn get_game_by_id(req: &mut Request) -> IronResult<Response> {
    unimplemented!()
}
fn get_games(req: &mut Request) -> IronResult<Response> {
    unimplemented!()
}
fn get_entry(req: &mut Request) -> IronResult<Response> {
    unimplemented!()
}
fn get_library(req: &mut Request) -> IronResult<Response> {
    unimplemented!()
}
fn get_user_by_id(req: &mut Request) -> IronResult<Response> {
    unimplemented!()
}
fn get_users(req: &mut Request) -> IronResult<Response> {
    unimplemented!()
}
