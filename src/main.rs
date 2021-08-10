use rocket;

mod state;
mod token;

#[rocket::catch(404)]
fn not_found(req: &rocket::Request) -> String {
    format!("Invalid path: {}", req.uri())
}

#[rocket::get("/ping")]
fn ping() -> &'static str {
    "OK"
}

#[rocket::main]
async fn main() {
    let solana_client = match state::solana_client_from_env() {
        Ok(client) => client,
        Err(e) => panic!("{:?}", e),
    };

    if let Err(e) = rocket::build()
        .manage(solana_client)
        .mount("/", rocket::routes![ping])
        .mount("/token", rocket::routes![token::check])
        .register("/", rocket::catchers![not_found])
        .launch()
        .await
    {
        println!("Could not launch server:");
        drop(e);
    }
}
