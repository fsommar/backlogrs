extern crate "rustc-serialize" as rustc_serialize;
use {Row, FromSqlRow};

#[derive(RustcEncodable, RustcDecodable)]
pub struct Person {
    pub name: String,
    pub age: i32
}

impl FromSqlRow for Person {
    fn from_sql_row<'stmt>(row: &Row<'stmt>) -> Person {
        Person {
            name: row.get(0),
            age: row.get(1)
        }
    }
}
