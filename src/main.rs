#[macro_use]
extern crate rocket;

use rocket::Request;

#[catch(404)]
fn not_found(req: &Request) -> String {
    format!("Invalid path: {}", req.uri())
}

#[get("/ping")]
fn ping() -> &'static str {
    "OK"
}

#[launch]
fn serve() -> _ {
    rocket::build()
        .mount("/", routes![ping])
        .register("/", catchers![not_found])
}
