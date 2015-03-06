extern crate "rustc-serialize" as serialize;
extern crate postgres;
extern crate time;
extern crate chrono;
use {Row, FromSqlRow};
use postgres::types::{self, Type};
use std::string;
use std::str::{self, FromStr};
use std::io::{Read, Write};

#[derive(Debug, Clone)]
pub struct UtcString(chrono::DateTime<chrono::UTC>);

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct Login {
    pub id: Option<i32>,
    pub username: String,
    pub password: String,
    pub email: String,
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct User {
    pub id: Option<i32>,
    pub username: String,
    pub email: String,
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct Library {
    pub id: Option<i32>,
    pub login_id: i32,
    pub entry_id: i32,
    pub user: Option<User>,
    pub entry: Option<Entry>,
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct Entry {
    pub id: Option<i32>,
    pub game_id: Option<i32>,
    pub time_played: Option<f32>,
    pub last_update: Option<String>,
    pub status: Option<Status>,
    pub game: Option<Game>,
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone, Copy)]
pub enum Status {
    Frozen,
    CurrentlyPlaying,
    Dropped,
    PlanToPlay,
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct Game {
    pub id: Option<i32>,
    pub name: String,
    pub description: String,
}

impl serialize::Encodable for UtcString {
    fn encode<S: serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_str(&self.to_string())
    }
}

impl serialize::Decodable for UtcString {
    fn decode<D: serialize::Decoder>(d: &mut D) -> Result<Self, D::Error> {
        let s = try!(d.read_str());
        Ok(FromStr::from_str(&s).unwrap())
    }
}

impl string::ToString for UtcString {
    fn to_string(&self) -> String {
        format!("{:?}", self.0)
    }
}

impl FromStr for UtcString {
    type Err = chrono::format::ParseError;

    fn from_str(s: &str) -> Result<UtcString, chrono::format::ParseError> {
        Ok(UtcString(try!(FromStr::from_str(s))))
    }
}

impl postgres::FromSql for UtcString {
    accepts!(Type::Timestamp, Type::TimestampTZ);

    fn from_sql<R: Read>(ty: &Type, raw: &mut R) -> postgres::Result<Self> {
        let ts: time::Timespec = postgres::FromSql::from_sql(ty, raw).unwrap();
        let dt = chrono::NaiveDateTime::from_timestamp(ts.sec, ts.nsec as u32);
        Ok(UtcString(chrono::DateTime::from_utc(dt, chrono::UTC)))
    }
}

impl postgres::ToSql for UtcString {
    accepts!(Type::Timestamp, Type::TimestampTZ);
    to_sql_checked!();

    fn to_sql<W: Write+?Sized>(&self, _: &Type, out: &mut W) -> postgres::Result<types::IsNull> {
        self.0.timestamp().to_sql(&Type::Int8, out)
    }
}

impl postgres::FromSql for Status {
    fn accepts(ty: &Type) -> bool {
        if let &Type::Other(ref o) = ty {
            o.name() == "status"
        } else {
            false
        }
    }

    fn from_sql<R: Read>(_: &Type, raw: &mut R) -> postgres::Result<Self> {
        let mut buf = vec![];
        try!(raw.read_to_end(&mut buf));
        let res = match str::from_utf8(&buf) {
            Ok("Frozen") => Status::Frozen,
            Ok("CurrentlyPlaying") => Status::CurrentlyPlaying,
            Ok("Dropped") => Status::Dropped,
            Ok("PlanToPlay") => Status::PlanToPlay,
            _ => return Err(postgres::Error::WasNull),
        };
        Ok(res)
    }
}

impl postgres::ToSql for Status {
    to_sql_checked!();

    fn accepts(ty: &Type) -> bool {
        if let &Type::Other(ref o) = ty {
            o.name() == "status"
        } else {
            false
        }
    }

    fn to_sql<W: Write+?Sized>(&self, _: &Type, out: &mut W) -> postgres::Result<types::IsNull> {
        let s = match *self {
            Status::Frozen => "Frozen",
            Status::CurrentlyPlaying => "CurrentlyPlaying",
            Status::Dropped => "Dropped",
            Status::PlanToPlay => "PlanToPlay",
        };
        try!(out.write(s.as_bytes()));
        Ok(types::IsNull::No)
    }
}

impl FromSqlRow for User {
    fn from_sql_row<'stmt>(row: &Row<'stmt>) -> User {
        User {
            id: Some(row.get(0)),
            username: row.get(1),
            email: row.get(3),
        }
    }
}

impl FromSqlRow for Login {
    fn from_sql_row<'stmt>(row: &Row<'stmt>) -> Login {
        Login {
            id: Some(row.get(0)),
            username: row.get(1),
            password: row.get(2),
            email: row.get(3),
        }
    }
}

impl FromSqlRow for Library {
    fn from_sql_row<'stmt>(row: &Row<'stmt>) -> Library {
        Library {
            id: Some(row.get(0)),
            login_id: row.get(1),
            entry_id: row.get(2),
            user: None,
            entry: None,
        }
    }
}

impl FromSqlRow for Entry {
    fn from_sql_row<'stmt>(row: &Row<'stmt>) -> Entry {
        let us: UtcString = row.get(3);
        Entry {
            id: Some(row.get(0)),
            game_id: Some(row.get(1)),
            time_played: row.get(2),
            last_update: Some(us.to_string()),
            status: Some(row.get(4)),
            game: None,
        }
    }
}

impl FromSqlRow for Game {
    fn from_sql_row<'stmt>(row: &Row<'stmt>) -> Game {
        Game {
            id: Some(row.get(0)),
            name: row.get(1),
            description: row.get(2),
        }
    }
}
