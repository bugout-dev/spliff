use super::state::SolanaClient;
use rocket::State;
use rocket::{form::name::Key, serde::json::Json};
use serde::{Deserialize, Serialize};
use serde_json;
use solana_client::rpc_request::TokenAccountsFilter;
use solana_sdk::{
    program_pack::Pack,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};

use spl_token::{self, instruction::initialize_mint, state::Mint};

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

fn new_throwaway_signer() -> (Keypair, Pubkey) {
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey();
    (keypair, pubkey)
}

#[derive(Deserialize, Debug)]
pub struct TokenSupply {
    supply: u64,
    decimals: u8,
}
#[rocket::post("/", data = "<token_supply_json>")]
pub fn create_token(
    token_supply_json: Json<TokenSupply>,
    solana_client: &State<SolanaClient>,
) -> String {
    let token_supply = token_supply_json.into_inner();
    let (token_signer, token) = new_throwaway_signer();
    // Also spl_token::initialize_accaut()
    let rent_exempt_fee = match solana_client
        .client
        .get_minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN)
    {
        Ok(fee) => fee,
        Err(_) => panic!("errror while getting rent exemp fee"),
    };

    let create_token_account_instruction = solana_sdk::system_instruction::create_account(
        &solana_client.pubkey, //fee payer
        &token,
        rent_exempt_fee,
        Mint::LEN as u64,
        &spl_token::id(), //owner
    );

    let mint_token_instruction = match initialize_mint(
        &spl_token::id(),
        &token,
        &solana_client.pubkey,
        None,
        token_supply.decimals,
    ) {
        Ok(instruction) => instruction,
        Err(_) => panic!("Error creating mint instruction"),
    };

    let instructions = vec![create_token_account_instruction, mint_token_instruction];

    let (recent_blockhash, _fee_calculator) = match solana_client.client.get_recent_blockhash() {
        Ok(result) => result,
        Err(_) => panic!("Could not get recent blockhash from Solana API"),
    };

    let create_token_transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&solana_client.pubkey),
        &vec![&solana_client.keypair, &token_signer],
        recent_blockhash,
    );

    let tx_signature = match solana_client
        .client
        .send_and_confirm_transaction(&create_token_transaction)
    {
        Ok(signature) => signature.to_string(),
        Err(_) => panic!("Error submitting transaction to Solana blockchain"),
    };

    return tx_signature;
}

// #[derive(Deserialize, Debug)]
// pub struct TokenTransferRequest {
//     token: String,
//     recipient: String,
//     amount: u64,
// }
// #[rocket::post("/transfer", data = "<token_transfer_request_json>")]
// pub fn transfer_token(
//     token_transfer_request_json: Json<TokenTransferRequest>,
//     solana_client: &State<SolanaClient>,
// ) {
//     println!("{:?}", token_transfer_request_json.into_inner());

//     let token_transfer_request = token_transfer_request_json.into_inner();
//     let token = Pubkey::new(token_transfer_request.token.as_bytes());
//     let destination = Pubkey::new(token_transfer_request.recipient.as_bytes());

//     //solana_sdk::system_instruction::transfer(from_pubkey: &Pubkey, to_pubkey: &Pubkey, lamports: u64)

//     spl_token::instruction::transfer(
//         &token,                //Token program id
//         &solana_client.pubkey, //source_pubkey
//         &destination,          //Destination pubkey
//         authority_pubkey: &Pubkey,
//         signer_pubkeys: &[&Pubkey],
//         token_transfer_request.amount, //amount
//     );
//     unimplemented!();
// }
