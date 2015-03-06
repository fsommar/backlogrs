extern crate iron;
extern crate router;
extern crate bodyparser;
#[macro_use] extern crate backlogrs;
extern crate time;
extern crate "rustc-serialize" as rustc_serialize;

use backlogrs::*;
use backlogrs::models::*;
use iron::prelude::*;
use router::Router;

fn main() {
    let mut router = Router::new();
    router.get("/user", get_users);
    // Users aren't allowed to update; as soon as a user
    // is created it is stuck that way. At least for now.
    router.post("/user", post_login);
    router.get("/user/:id", get_user_by_id);
    router.get("/user/:id/library", get_library);
    router.get("/user/:uid/library/:eid", get_entry);
    router.post("/user/:id/library", post_entry);
    router.get("/game", get_games);
    router.get("/game/:id", get_game_by_id);
    router.get("/status", get_status);

    let mut chain = Chain::new(router);
    chain.link_before(Api);
    chain.link_before(DbConnection::new());
        // Prints the error in html body
        chain.link_after(DebugIronError);

        println!("Listening on port 3000...");
        Iron::new(chain).http("0.0.0.0:3000").unwrap();
    }

    fn post_entry(req: &mut Request) -> IronResult<Response> {
        let e = status::BadRequest;
        let mut new_entry = try!(req.get::<bodyparser::Struct<Entry>>()
                                .on_err(e)).unwrap();
        let user_id = try!(req.get_from_router::<i32>("id").on_err(e));

        let db = req.db();
        if let Some(entry_id) = new_entry.id {
            // The entry should be updated
            let stmt = try_iron!(db.prepare(
                    "SELECT e.* FROM Login lo JOIN Library li ON lo.id = li.login_id \
                    JOIN Entry e ON e.id = li.entry_id WHERE e.id = $1 AND lo.id = $2"));
            let prev_entry = try_iron!(stmt.query(&[&entry_id, &user_id]))
                .collect_sql::<Vec<Entry>>()[0].clone();

            if new_entry.status.is_none() {
                new_entry.status = prev_entry.status;
            }
            if new_entry.time_played.is_none() {
                new_entry.time_played = prev_entry.time_played;
            }

            try_iron!(db.execute(
                    "UPDATE Entry e SET status = $3, time_played = $4 WHERE e.id = $1 AND EXISTS \
                    (SELECT * FROM Library li WHERE li.entry_id = $1 AND li.login_id = $2)",
                    &[&entry_id, &user_id, &new_entry.status.unwrap(), &new_entry.time_played])
                => "failed, probably because name/email already exists");
        } else {
            if new_entry.game_id.is_none() {
                return Err(LibError::Cause(
                        "Both game and entry ID can't be null!".to_string())).on_err(e);
            }
            if new_entry.status.is_none() {
                new_entry.status = Some(Status::PlanToPlay);
            }
            if new_entry.time_played.is_none() {
                new_entry.time_played = Some(0.0);
            }

            // Insert new entry
            // Create transaction and commit if everything went as expected
            let trans = try_iron!(db.transaction());
            // First create entry and then map that into a library
            let stmt = try_iron!(trans.prepare(
                    "INSERT INTO Entry (game_id, time_played, status) \
                    VALUES ($1, $2, $3) RETURNING *"));
            new_entry = try_iron!(opt: try_iron!(
                    stmt.query(&[&new_entry.game_id, &new_entry.time_played,
                            &new_entry.status.unwrap()]))
                .collect_sql::<Vec<Entry>>().pop()
                => "Failed inserting new entry");

            // Create library entry
            try_iron!(trans.execute(
                    "INSERT INTO Library (entry_id, login_id) VALUES ($1, $2)",
                    &[&new_entry.id.unwrap(), &user_id]));

            try_iron!(trans.commit());
        }

        Ok(Response::with((status::Ok, Json(new_entry))))
    }

    fn post_login(req: &mut Request) -> IronResult<Response> {
        let login = try!(req.get::<bodyparser::Struct<Login>>()
                        .on_err(status::BadRequest)).unwrap();

        let db = req.db();
        let stmt = try_iron!(db.prepare(
                "INSERT INTO Login (username, password, email) \
                VALUES ($1, $2, $3) RETURNING id, username, NULL, email"));
        let user = try_iron!(stmt.query(
                &[&login.username, &login.password, &login.email]))
            .collect_sql::<Vec<User>>().pop();

        Ok(Response::with((status::Ok, Json(user))))
    }

    fn get_status(req: &mut Request) -> IronResult<Response> {
        let db = req.db();
        let stmt = try_iron!(db.prepare("SELECT unnest(enum_range(NULL::Status))"));
        let res = try_iron!(stmt.query(&[])).iter().map(|x| {
            x.get(0)
        }).collect::<Vec<Status>>();

        Ok(Response::with((status::Ok, Json(res))))
    }

    fn get_game_by_id(req: &mut Request) -> IronResult<Response> {
        let id = try!(req.get_from_router::<i32>("id")
                    .on_err(status::BadRequest));

        let db = req.db();
        let stmt = try_iron!(db.prepare("SELECT * FROM Game WHERE id = $1"));
        let mut res = try_iron!(stmt.query(&[&id])).collect_sql::<Vec<Game>>();

        if res.is_empty() {
            Ok(Response::with(status::NoContent))
        } else {
            Ok(Response::with((status::Ok, Json(res.pop()))))
        }
    }

    fn get_games(req: &mut Request) -> IronResult<Response> {
        let db = req.db();
        let stmt = try_iron!(db.prepare("SELECT * FROM Game ORDER BY name"));
        let res = try_iron!(stmt.query(&[])).collect_sql::<Vec<Game>>();

        Ok(Response::with((status::Ok, Json(res))))
    }

    fn get_entry(req: &mut Request) -> IronResult<Response> {
        let e = status::BadRequest;
        let user_id = try!(req.get_from_router::<i32>("uid").on_err(e));
        let entry_id = try!(req.get_from_router::<i32>("eid").on_err(e));

        let db = req.db();
        let stmt = try_iron!(db.prepare(
                "SELECT e.* FROM Login lo JOIN Library li ON lo.id = li.login_id \
                JOIN Entry e ON e.id = li.entry_id WHERE e.id = $1 AND lo.id = $2"));
        let mut res = try_iron!(stmt.query(&[&entry_id, &user_id]))
            .collect_sql::<Vec<Entry>>();

        if res.is_empty() {
            Ok(Response::with(status::NoContent))
        } else {
            Ok(Response::with((status::Ok, Json(res.pop()))))
        }
    }

    fn get_library(req: &mut Request) -> IronResult<Response> {
        let user_id = try!(req.get_from_router::<i32>("id")
                        .on_err(status::BadRequest));

        let db = req.db();
        let stmt = try_iron!(db.prepare(
                "SELECT e.* FROM Login lo JOIN Library li ON lo.id = li.login_id \
                JOIN Entry e ON e.id = li.entry_id WHERE lo.id = $1"));
    let res = try_iron!(stmt.query(&[&user_id]))
        .collect_sql::<Vec<Entry>>();

    if res.is_empty() {
        Ok(Response::with(status::NoContent))
    } else {
        Ok(Response::with((status::Ok, Json(res))))
    }
}

fn get_user_by_id(req: &mut Request) -> IronResult<Response> {
    let id = try!(req.get_from_router::<i32>("id")
                  .on_err(status::BadRequest));

    let db = req.db();
    let stmt = try_iron!(db.prepare("SELECT * FROM Login WHERE id = $1"));
    let mut res = try_iron!(stmt.query(&[&id])).collect_sql::<Vec<User>>();

    if res.is_empty() {
        Ok(Response::with(status::NoContent))
    } else {
        Ok(Response::with((status::Ok, Json(res.pop()))))
    }
}

fn get_users(req: &mut Request) -> IronResult<Response> {
    let db = req.db();
    let stmt = try_iron!(db.prepare("SELECT * FROM Login"));
    let res = try_iron!(stmt.query(&[])).collect_sql::<Vec<User>>();

    Ok(Response::with((status::Ok, Json(res))))
}
