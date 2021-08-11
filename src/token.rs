use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use serde_json;
use solana_client::rpc_request::TokenAccountsFilter;
use solana_sdk::{
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
};
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

fn new_throwaway_signer() -> (Box<dyn Signer>, Pubkey) {
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey();
    (Box::new(keypair) as Box<dyn Signer>, pubkey)
}

#[derive(Deserialize, Debug)]
pub struct TokenSuply {
    supply: u64,
}
#[rocket::post("/", data = "<token_supply>")]
pub fn create_token(token_supply: Json<TokenSuply>, solana_client: &State<SolanaClient>) {
    println!("{:?}", token_supply.into_inner());
    let (token_signer, token) = new_throwaway_signer();
    // Also spl_token::initialize_accaut()
    solana_sdk::system_instruction::create_account(
        from_pubkey: &Pubkey, //fee payer
        &token,
        lamports: u64,
        space: u64,
        &spl_token::id(), //owner
    );
    unimplemented!();
}

#[derive(Deserialize, Debug)]
pub struct TokenTransferRequest {
    token: String,
    recipient: String,
    amount: u64,
}
#[rocket::post("/transfer", data = "<token_transfer_request_json>")]
pub fn transfer_token(
    token_transfer_request_json: Json<TokenTransferRequest>,
    solana_client: &State<SolanaClient>,
) {
    println!("{:?}", token_transfer_request_json.into_inner());

    let token_transfer_request = token_transfer_request_json.into_inner();
    let token = Pubkey::new(token_transfer_request.token.as_bytes());
    let destination = Pubkey::new(token_transfer_request.recipient.as_bytes());

    //solana_sdk::system_instruction::transfer(from_pubkey: &Pubkey, to_pubkey: &Pubkey, lamports: u64)

    spl_token::instruction::transfer(
        &token,                //Token program id
        &solana_client.pubkey, //source_pubkey
        &destination,          //Destination pubkey
        authority_pubkey: &Pubkey,
        signer_pubkeys: &[&Pubkey],
        token_transfer_request.amount, //amount
    );
    unimplemented!();
}
