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

#[derive(Deserialize, Serialize, Debug)]
pub struct ListTokensResponse {
    tokens: Vec<TokenBalance>,
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
