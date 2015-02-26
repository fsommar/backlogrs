extern crate iron;
extern crate router;
extern crate bodyparser;
extern crate backlogrs;
extern crate time;
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
    // Users aren't allowed to update; as soon as a user
    // is created it is stuck that way. At least for now.
    router.post("/user", post_login);
    router.get("/user/:id", get_user_by_id);
    router.get("/user/:id/library", get_library);
    router.get("/user/:uid/library/:eid", get_entry);
    router.post("/user/:uid/library", post_entry);
    router.get("/game", get_games);
    router.get("/game/:id", get_game_by_id);
    router.get("/status", get_status);

    let mut chain = Chain::new(router);
    chain.link_before(Api);
    chain.around(DbConnection::new());
    // Prints the error in html body
    chain.link_after(DebugIronError);

    println!("Listening on port 3000...");
    Iron::new(chain).http("0.0.0.0:3000").unwrap();
}

fn post_entry(req: &mut Request) -> IronResult<Response> {
    let mut new_entry = try!(req.get::<bodyparser::Struct<Entry>>()
                         .on_err("bad request")).unwrap();
    let user_id: i32 = FromStr::from_str(req.extensions.find::<Router>().unwrap().find("uid").unwrap()).unwrap();
    let e = status::InternalServerError;

    let db = req.db();
    if let Some(entry_id) = new_entry.id {
        // The entry should be updated
        let stmt = try!(db.prepare(
                "SELECT e.* FROM Login lo JOIN Library li ON lo.id = li.login_id \
                JOIN Entry e ON e.id = li.entry_id WHERE e.id = $1 AND lo.id = $2")
            .on_err(e));
        let prev_entry = try!(stmt.query(&[&entry_id, &user_id]).on_err(e))
            .collect_sql::<Vec<Entry>>()[0].clone();

        if new_entry.status.is_none() {
            new_entry.status = prev_entry.status;
        }
        if new_entry.time_played.is_none() {
            new_entry.time_played = prev_entry.time_played;
        }

        try!(db.execute(
                "UPDATE Entry e SET status = $3, time_played = $4 WHERE e.id = $1 AND EXISTS \
                (SELECT * FROM Library li WHERE li.entry_id = $1 AND li.login_id = $2)",
                &[&entry_id, &user_id, &new_entry.status.unwrap(), &new_entry.time_played])
            .on_err(
                "failed, probably because name/email already exists"));
    } else {
        if new_entry.game_id.is_none() {
            return Err(IronError::new(LibError, "test!"));
        }
        if new_entry.status.is_none() {
            new_entry.status = Some(Status::PlanToPlay);
        }
        if new_entry.time_played.is_none() {
            new_entry.time_played = Some(0.0);
        }

        // Insert new entry
        // Create transaction and commit if everything went as expected
        let trans = try!(db.transaction().on_err("failed getting transaction"));
        // First create entry and then map that into a library
        let stmt = try!(trans.prepare(
                "INSERT INTO Entry (game_id, time_played, status) \
                VALUES ($1, $2, $3) RETURNING id")
            .on_err(e));
        let entry_id: i32 = try!(stmt.query(
                &[&new_entry.game_id, &new_entry.time_played, &new_entry.status.unwrap()])
            .on_err(e)).next().unwrap().get(0);

        // Create library entry
        try!(trans.execute(
                "INSERT INTO Library (entry_id, login_id) VALUES ($1, $2)",
                &[&entry_id, &user_id])
            .on_err(e));

        try!(trans.commit().on_err(e));
    }

    Ok(Response::with((status::Ok, Json(new_entry))))
}

fn post_login(req: &mut Request) -> IronResult<Response> {
    let login = try!(req.get::<bodyparser::Struct<Login>>()
                     .on_err("bad request")).unwrap();

    let db = req.db();

    try!(db.execute(
            "INSERT INTO Login (username, password, email) \
            VALUES ($1, $2, $3)",
            &[&login.username,
            &login.password,
            &login.email]).on_err(
                "failed, probably because name/email already exists"));

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
    // TODO: Create an extension method to do this prettier
    let id: i32 = req.extensions.find::<Router>()
        .and_then(|x| x.find("id"))
        .and_then(|x| FromStr::from_str(x).ok()).unwrap();
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
    let query_str = req.extensions.find::<Router>().unwrap().find("id").unwrap_or("-1");
    let user_id: i32 = FromStr::from_str(query_str).unwrap();
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
    let query_str = req.extensions.find::<Router>().and_then(|x| x.find("id")).unwrap_or("");
    println!("query_str: {:?}", query_str);

    let id: i32 = FromStr::from_str(query_str).unwrap();
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
