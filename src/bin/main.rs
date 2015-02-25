extern crate iron;
extern crate router;
extern crate bodyparser;
extern crate backlogrs;
extern crate "rustc-serialize" as rustc_serialize;

use backlogrs::*;
use backlogrs::models::*;
use iron::prelude::*;
use iron::{status};
use router::Router;
use rustc_serialize::json;
use std::str::FromStr;
use std::old_io::BufferedReader;
use std::old_io::ByRefReader;

struct DebugIronError;

impl iron::AfterMiddleware for DebugIronError {
    fn catch(&self, req: &mut Request, err: IronError) -> IronResult<Response> {
        // In case of error bubbling up
        Ok(Response::with(format!("{:?}", err)))
    }
}

fn main() {
    let mut router = Router::new();
    router.get("/user", get_users);
    router.post("/user", post_login);
    router.get("/user/:id", get_user_by_id);
    router.get("/user/:id/library", get_library);
    router.get("/user/:uid/library/:eid", get_entry);
    router.get("/game", get_games);
    router.get("/game/:id", get_game_by_id);
    router.get("/status", get_status);

    let mut chain = Chain::new(router);
    chain.link_before(Api);
    chain.around(DbConnection::new());
    // Prints the error in html body
    chain.link_after(DebugIronError);

    Iron::new(chain).http("0.0.0.0:3000").unwrap();
    println!("Listening on port 3000...");
}

fn post_login(req: &mut Request) -> IronResult<Response> {
    let login = try!(req.get::<bodyparser::Struct<Login>>()
                     .on_err("bad request")).unwrap();

    let db = req.db();
    let e = status::InternalServerError;

    let stmt = try!(db.prepare(
            "INSERT INTO Login (username, password, email) \
            VALUES ($1, $2, $3)").on_err(e));
    try!(stmt.query(&[&login.username,
                    &login.password,
                    &login.email]).on_err(e));

    Ok(Response::with((status::Ok, Json(login))))
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
    let db = req.db();
    let id: i32 = FromStr::from_str(req.extensions.find::<Router>().unwrap().find("id").unwrap()).unwrap();
    let e = status::InternalServerError;

    let stmt = try!(db.prepare("SELECT * FROM Game WHERE id = $1").on_err(e));
    let res = try!(stmt.query(&[&id]).on_err(("Oops", e))).collect_sql::<Vec<Game>>();

    Ok(Response::with((status::Ok, Json(res))))
}

fn get_games(req: &mut Request) -> IronResult<Response> {
    let db = req.db();
    let e = ("Database error!", status::InternalServerError);

    let stmt = try!(db.prepare("SELECT * FROM Game").on_err(e));
    let res = try!(stmt.query(&[]).on_err(e)).collect_sql::<Vec<Game>>();

    Ok(Response::with((status::Ok, Json(res))))
}

fn get_entry(req: &mut Request) -> IronResult<Response> {
    let db = req.db();
    let user_id: i32 = FromStr::from_str(req.extensions.find::<Router>().unwrap().find("uid").unwrap()).unwrap();
    let entry_id: i32 = FromStr::from_str(req.extensions.find::<Router>().unwrap().find("eid").unwrap()).unwrap();
    let e = status::InternalServerError;

    let stmt = try!(db.prepare(
            "SELECT e.* FROM Login lo JOIN Library li ON lo.id = li.login_id \
            JOIN Entry e ON e.id = li.entry_id WHERE e.id = $1 AND lo.id = $2").on_err(e));
    let res = try!(stmt.query(&[&entry_id, &user_id]).on_err(("Oops", e)))
        .collect_sql::<Vec<Entry>>();

    Ok(Response::with((status::Ok, Json(res))))
}

fn get_library(req: &mut Request) -> IronResult<Response> {
    let db = req.db();
    let user_id: i32 = FromStr::from_str(req.extensions.find::<Router>().unwrap().find("id").unwrap()).unwrap();
    let e = status::InternalServerError;

    let stmt = try!(db.prepare(
            "SELECT e.* FROM Login lo JOIN Library li ON lo.id = li.login_id \
            JOIN Entry e ON e.id = li.entry_id WHERE lo.id = $1").on_err(e));
    let res = try!(stmt.query(&[&user_id]).on_err(("Oops", e)))
        .collect_sql::<Vec<Entry>>();

    Ok(Response::with((status::Ok, Json(res))))
}

fn get_user_by_id(req: &mut Request) -> IronResult<Response> {
    let db = req.db();
    let id: i32 = FromStr::from_str(req.extensions.find::<Router>().unwrap().find("id").unwrap()).unwrap();
    let e = status::InternalServerError;

    let stmt = try!(db.prepare("SELECT * FROM Login WHERE id = $1").on_err(e));
    let res = try!(stmt.query(&[&id]).on_err(("Oops", e))).collect_sql::<Vec<User>>();

    Ok(Response::with((status::Ok, Json(res))))
}

fn get_users(req: &mut Request) -> IronResult<Response> {
    let db = req.db();
    let e = ("Database error!", status::InternalServerError);

    let stmt = try!(db.prepare("SELECT * FROM Login").on_err(e));
    let res = try!(stmt.query(&[]).on_err(e)).collect_sql::<Vec<User>>();

    Ok(Response::with((status::Ok, Json(res))))
}
