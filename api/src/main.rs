use rocket;

mod token;
use spliff_lib::{self, state::SolanaClient};
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
    println!("{:?}", spliff_lib::hello());

    let solana_client = match SolanaClient::from_env() {
        Ok(client) => client,
        Err(e) => panic!("Error while initializing solana client: {:?}", e),
    };

    println!(
        "Running server using keypair with public key: {:?}",
        solana_client.pubkey
    );

    if let Err(e) = rocket::build()
        .manage(solana_client)
        .mount("/", rocket::routes![ping])
        .mount(
            "/tokens",
            rocket::routes![
                token::list_tokens,
                token::create_token,
                token::transfer_token,
            ],
        )
        .register("/", rocket::catchers![not_found])
        .launch()
        .await
    {
        println!("Could not launch server:");
        drop(e);
    }
}
