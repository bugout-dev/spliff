use rocket::State;

use super::state::SolanaClient;

#[rocket::get("/check")]
pub fn check(solana_client: &State<SolanaClient>) -> Result<String, String> {
    match solana_client.client.get_block_height() {
        Ok(height) => Ok(height.to_string()),
        Err(_) => Err(String::from("oh no")),
    }
}
