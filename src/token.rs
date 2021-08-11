use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use serde_json;
use solana_client::rpc_request::TokenAccountsFilter;
use spl_token;

use super::state::SolanaClient;

#[derive(Deserialize, Serialize, Debug)]
pub struct TokenBalance {
    token: String,
    pubkey: String,
}

#[rocket::get("/")]
pub fn list_tokens(solana_client: &State<SolanaClient>) -> String {
    let accounts: Vec<TokenBalance> = match solana_client.client.get_token_accounts_by_owner(
        &solana_client.pubkey,
        TokenAccountsFilter::ProgramId(spl_token::id()),
    ) {
        Ok(results) => results
            .iter()
            .map(|token_account| TokenBalance {
                token: token_account.pubkey.to_string(),
                pubkey: solana_client.pubkey.to_string(),
            })
            .collect(),
        Err(_) => vec![],
    };

    return serde_json::json!(accounts).to_string();
}

#[derive(Deserialize, Debug)]
pub struct TokenSuply {
    supply: u64,
}
#[rocket::post("/", data = "<token_supply>")]
pub fn create_token(token_supply: Json<TokenSuply>, solana_client: &State<SolanaClient>) {
    println!("{:?}", token_supply.into_inner());
    unimplemented!();
}

#[derive(Deserialize, Debug)]
pub struct TokenTransferRequest {
    token: String,
    recipient: String,
    amount: u64,
}
#[rocket::post("/transfer", data = "<token_transfer_request>")]
pub fn transfer_token(token_transfer_request: Json<TokenTransferRequest>) {
    println!("{:?}", token_transfer_request.into_inner());
    unimplemented!();
}
