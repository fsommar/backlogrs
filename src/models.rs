extern crate "rustc-serialize" as rustc_serialize;
extern crate postgres;
use {Row, FromSqlRow};
use postgres::types::Type;

#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub email: String,
}

#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct Library {
    pub id: i32,
    pub user_id: i32,
    pub entry_id: i32,
    pub user: Option<User>,
    pub entry: Option<Entry>,
}

#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct Entry {
    pub id: i32,
    pub game_id: i32,
    pub time_played: f32,
    pub last_update: i64,// date
    pub status: Status,
    pub game: Option<Game>,
}

#[derive(RustcEncodable, RustcDecodable, Debug)]
pub enum Status {
    Frozen,
    CurrentlyPlaying,
    Dropped,
    PlanToPlay,
}

#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct Game {
    pub id: i32,
    pub name: String,
    pub description: String,
}

#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct Person {
    pub name: String,
    pub age: i32
}

impl postgres::FromSql for Status {
    fn from_sql(ty: &Type, raw: Option<&[u8]>) -> postgres::Result<Self> {
        let valid_type = match *ty {
            // Assume name always is lower case
            Type::Unknown(ref unk) => unk.name() == "status",
            _ => false,
        };
        if !valid_type {
            return Err(postgres::Error::WrongType(ty.clone()));
        }

        let err = Err(postgres::Error::BadData);
        if let Some(x) = raw {
            let res = match ::std::str::from_utf8(x).unwrap() {
                "Frozen" => Status::Frozen,
                "CurrentlyPlaying" => Status::CurrentlyPlaying,
                "Dropped" => Status::Dropped,
                "PlanToPlay" => Status::PlanToPlay,
                _ => return err,
            };
            Ok(res)
        } else {
            err
        }
    }
}

impl FromSqlRow for User {
    fn from_sql_row<'stmt>(row: &Row<'stmt>) -> User {
        User {
            id: row.get(0),
            username: row.get(1),
            password: row.get(2),
            email: row.get(3),
        }
    }
}

impl FromSqlRow for Library {
    fn from_sql_row<'stmt>(row: &Row<'stmt>) -> Library {
        Library {
            id: row.get(0),
            user_id: row.get(1),
            entry_id: row.get(2),
            user: None,
            entry: None,
        }
    }
}

impl FromSqlRow for Entry {
    fn from_sql_row<'stmt>(row: &Row<'stmt>) -> Entry {
        Entry {
            id: row.get(0),
            game_id: row.get(1),
            time_played: row.get(3),
            last_update: row.get(4),
            status: row.get(5),
            game: None,
        }
    }
}

impl FromSqlRow for Game {
    fn from_sql_row<'stmt>(row: &Row<'stmt>) -> Game {
        Game {
            id: row.get(0),
            name: row.get(1),
            description: row.get(2),
        }
    }
}

impl FromSqlRow for Person {
    fn from_sql_row<'stmt>(row: &Row<'stmt>) -> Person {
        Person {
            name: row.get(0),
            age: row.get(1)
        }
    }
}
