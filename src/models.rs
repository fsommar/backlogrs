extern crate "rustc-serialize" as rustc_serialize;

#[derive(RustcEncodable, RustcDecodable)]
pub struct Person {
    pub name: String,
    pub age: i32
}
