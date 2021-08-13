use super::state::SolanaClient;
use core::panic;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use serde_json;
use solana_client::rpc_request::TokenAccountsFilter;
use solana_sdk::{
    program_pack::Pack,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};
use spl_associated_token_account::*;
use std::str::FromStr;

use spl_token::instruction::mint_to_checked;
use spl_token::{self, instruction::initialize_mint, state::Mint};

#[derive(Deserialize, Serialize, Debug)]
pub struct TokenBalance {
    token_accaunt: String,
    balance: String,
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
            .map(|token_account| -> TokenBalance {
                let token_pubkey = Pubkey::from_str(&token_account.pubkey[..]).unwrap();
                let balance = match solana_client
                    .client
                    .get_token_account_balance(&token_pubkey)
                {
                    Ok(res) => res,
                    Err(_) => panic!("Couldnt get token balance"),
                };
                TokenBalance {
                    token_accaunt: token_account.pubkey.to_string(),
                    balance: balance.amount,
                    pubkey: solana_client.pubkey.to_string(),
                }
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
    supply: f64,
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
        Err(err) => panic!(
            "Error submitting transaction to Solana blockchain: {:?}",
            err
        ),
    };

    println!("{}", token);
    create_token_account(&solana_client, &token);
    mint_token(solana_client, &token, &token_signer, &token_supply);

    return format!("Successfully created token");
}

fn mint_token(
    solana_client: &SolanaClient,
    token: &Pubkey,
    token_signer: &Keypair,
    token_supply: &TokenSupply,
) {
    let account = get_associated_token_address(&solana_client.pubkey, &token);
    let mint_amount = spl_token::ui_amount_to_amount(token_supply.supply, token_supply.decimals);

    let mint_supply_instruction = match mint_to_checked(
        &spl_token::id(),
        &token,
        &account,
        &solana_client.pubkey,
        &vec![&solana_client.pubkey, &token],
        mint_amount,
        token_supply.decimals,
    ) {
        Ok(res) => res,
        Err(err) => panic!("Error while minting: {:?}", err),
    };

    let (recent_blockhash, _fee_calculator) = match solana_client.client.get_recent_blockhash() {
        Ok(result) => result,
        Err(_) => panic!("Could not get recent blockhash from Solana API"),
    };

    let mint_supply_transaction = Transaction::new_signed_with_payer(
        &vec![mint_supply_instruction],
        Some(&solana_client.pubkey),
        &vec![&solana_client.keypair, &token_signer],
        recent_blockhash,
    );

    let tx_signature_for_mint = match solana_client
        .client
        .send_and_confirm_transaction(&mint_supply_transaction)
    {
        Ok(signature) => signature.to_string(),
        Err(err) => panic!("Error while minting supply transaction: {:?}", err),
    };
}

fn create_token_account(solana_client: &SolanaClient, token: &Pubkey) {
    //TODO(yhtiyar):
    //It is good idea to check if account already exists
    //before trying to create one
    //let account = get_associated_token_address(&solana_client.pubkey, &token);

    let instructions = vec![create_associated_token_account(
        &solana_client.pubkey, //Funding address
        &solana_client.pubkey, //Wallet address
        &token,
    )];

    let (recent_blockhash, _fee_calculator) = match solana_client.client.get_recent_blockhash() {
        Ok(result) => result,
        Err(_) => panic!("Could not get recent blockhash from Solana API"),
    };

    let create_token_account_transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&solana_client.pubkey),
        &vec![&solana_client.keypair],
        recent_blockhash,
    );

    let tx_signature = match solana_client
        .client
        .send_and_confirm_transaction(&create_token_account_transaction)
    {
        Ok(signature) => signature.to_string(),
        Err(err) => panic!(
            "Error submitting create token accaunt transaction to Solana blockchain: {:?}",
            err
        ),
    };
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
