use super::state::SolanaClient;
use core::panic;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use serde_json;
use solana_account_decoder::{parse_token::TokenAccountType, UiAccountData};
use solana_client::{rpc_request::TokenAccountsFilter, rpc_response::RpcKeyedAccount};
use solana_sdk::system_program;
use solana_sdk::{
    program_pack::Pack,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};
use spl_associated_token_account::*;
use spl_token::instruction::mint_to_checked;
use spl_token::{self, instruction::initialize_mint, state::Mint};
use std::str::FromStr;

#[derive(Serialize, Debug)]
pub struct TokenBalance {
    token_address: String,
    token_account: String,
    balance: String,
    owner_pubkey: String,
}

fn parse_account(account: &RpcKeyedAccount, owner: &Pubkey) -> TokenBalance {
    if let UiAccountData::Json(parsed_account) = account.account.data.clone() {
        match serde_json::from_value(parsed_account.parsed) {
            Ok(TokenAccountType::Account(ui_token_account)) => {
                let mint = ui_token_account.mint.clone();
                return TokenBalance {
                    token_address: mint,
                    token_account: account.pubkey.clone(),
                    balance: ui_token_account.token_amount.real_number_string(),
                    owner_pubkey: owner.to_string(),
                };
            }
            Ok(_) => panic!("unsupported account type"),
            Err(err) => panic!("Error while parsing account {:?}", err),
        }
    } else {
        panic!("Failed to parse account")
    }
}

#[rocket::get("/")]
pub fn list_tokens(solana_client: &State<SolanaClient>) -> Result<Json<Vec<TokenBalance>>, Status> {
    let accounts: Vec<TokenBalance> = match solana_client.client.get_token_accounts_by_owner(
        &solana_client.pubkey,
        TokenAccountsFilter::ProgramId(spl_token::id()),
    ) {
        Ok(results) => results
            .iter()
            .map(|token_account| -> TokenBalance {
                parse_account(token_account, &solana_client.pubkey)
            })
            .collect(),
        Err(err) => {
            println!("{:?}", err);
            return Err(Status::InternalServerError);
        }
    };

    return Ok(Json(accounts));
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

#[derive(Serialize, Debug)]
pub struct CreateTokenResponse {
    token: String,
    token_account: String,
    balance: String,
    transaction: String,
}

#[rocket::post("/", data = "<token_supply_json>")]
pub fn create_token(
    token_supply_json: Json<TokenSupply>,
    solana_client: &State<SolanaClient>,
) -> Result<Json<CreateTokenResponse>, (Status, String)> {
    let token_supply = token_supply_json.into_inner();
    let (token_signer, token) = new_throwaway_signer();
    let rent_exempt_fee = match solana_client
        .client
        .get_minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN)
    {
        Ok(fee) => fee,
        Err(err) => {
            println!("{:?}", err);
            return Err((
                Status::InternalServerError,
                "Failed while calculating minimum balance for rent exemption blockhash".to_string(),
            ));
        }
    };

    let create_token_instruction = solana_sdk::system_instruction::create_account(
        &solana_client.pubkey, //fee payer
        &token,
        rent_exempt_fee,
        Mint::LEN as u64,
        &spl_token::id(), //owner
    );

    let initialize_mint_instruction = match initialize_mint(
        &spl_token::id(),
        &token,
        &solana_client.pubkey,
        None,
        token_supply.decimals,
    ) {
        Ok(instruction) => instruction,
        Err(err) => {
            println!("{:?}", err);
            return Err((
                Status::BadRequest,
                "Failied to make initialize mint instruction".to_string(),
            ));
        }
    };

    let create_token_account_instruction = create_associated_token_account(
        &solana_client.pubkey, //Funding address
        &solana_client.pubkey, //Wallet address
        &token,
    );

    let token_account = get_associated_token_address(&solana_client.pubkey, &token);
    let mint_amount = spl_token::ui_amount_to_amount(token_supply.supply, token_supply.decimals);

    let mint_supply_instruction = match mint_to_checked(
        &spl_token::id(),
        &token,
        &token_account,
        &solana_client.pubkey,
        &vec![&solana_client.pubkey, &token],
        mint_amount,
        token_supply.decimals,
    ) {
        Ok(res) => res,
        Err(err) => {
            println!("{:?}", err);
            return Err((
                Status::BadRequest,
                "Failed to make mint supply instructions".to_string(),
            ));
        }
    };

    let instructions = vec![
        create_token_instruction,
        initialize_mint_instruction,
        create_token_account_instruction,
        mint_supply_instruction,
    ];

    let (recent_blockhash, _fee_calculator) = match solana_client.client.get_recent_blockhash() {
        Ok(result) => result,
        Err(err) => {
            println!("{:?}", err);
            return Err((
                Status::BadRequest,
                "Failed to calculate recent blockhash".to_string(),
            ));
        }
    };

    let init_token_transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&solana_client.pubkey),
        &vec![&solana_client.keypair, &token_signer],
        recent_blockhash,
    );

    let tx_signature = match solana_client
        .client
        .send_and_confirm_transaction(&init_token_transaction)
    {
        Ok(signature) => signature.to_string(),
        Err(err) => {
            println!("{:?}", err);
            return Err((
                Status::InternalServerError,
                "Failed while excecuting transaction".to_string(),
            ));
        }
    };

    return Ok(Json(CreateTokenResponse {
        token: token.to_string(),
        token_account: token_account.to_string(),
        balance: mint_amount.to_string(),
        transaction: tx_signature,
    }));
}

fn has_token_account(
    solana_adress: &Pubkey,
    token: &Pubkey,
    solana_client: &SolanaClient,
) -> Result<bool, String> {
    let account = get_associated_token_address(&solana_adress, &token);
    let account_with_commitment = match solana_client
        .client
        .get_account_with_commitment(&account, solana_client.client.commitment())
    {
        Ok(acc) => acc,
        Err(e) => return Err(format!("{:?}", e)),
    };
    if let Some(account_data) = account_with_commitment.value {
        if !(account_data.owner == system_program::id()) {
            return Ok(true);
        }
    }
    return Ok(false);
}

#[derive(Deserialize, Debug)]
pub struct TokenTransferRequest {
    token: String,
    recipient: String, // Expected to be a token account.
    amount: u64,
}

#[derive(Serialize, Debug)]
pub struct TokenTransferResponse {
    token: String,
    recipient: String,
    amount: u64,
    transaction: String,
}

#[rocket::post("/transfer", data = "<token_transfer_request_json>")]
pub fn transfer_token(
    token_transfer_request_json: Json<TokenTransferRequest>,
    solana_client: &State<SolanaClient>,
) -> Result<Json<TokenTransferResponse>, (Status, String)> {
    let token_transfer_request = token_transfer_request_json.into_inner();
    let token_pubkey = match Pubkey::from_str(&token_transfer_request.token) {
        Ok(pubkey) => pubkey,
        Err(err) => {
            println!("{:?}", err);
            return Err((
                Status::BadRequest,
                format!(
                    "Failed to parse token address: {:?}",
                    &token_transfer_request.token
                ),
            ));
        }
    };
    let recipient_pubkey = match Pubkey::from_str(&token_transfer_request.recipient) {
        Ok(pubkey) => pubkey,
        Err(err) => {
            println!("{:?}", err);
            return Err((
                Status::BadRequest,
                format!(
                    "Failed to parse recipient address: {:?}",
                    &token_transfer_request.recipient,
                ),
            ));
        }
    };

    //solana_sdk::system_instruction::transfer(from_pubkey: &Pubkey, to_pubkey: &Pubkey, lamports: u64)

    let source_account = get_associated_token_address(&solana_client.pubkey, &token_pubkey);
    match has_token_account(&recipient_pubkey, &token_pubkey, &solana_client) {
        Ok(res) => {
            if !res {
                return Err((
                    Status::BadRequest,
                    format!(
                        "For {} address, token account for {} token not found",
                        &recipient_pubkey, &token_pubkey
                    ),
                ));
            }
        }
        Err(err) => {
            println!("{:?}", err);
            return Err((
                Status::InternalServerError,
                format!(
                    "Failed to get account details of {} address for {} token",
                    &recipient_pubkey, &token_pubkey
                ),
            ));
        }
    }
    let recipient_account = get_associated_token_address(&solana_client.pubkey, &token_pubkey);
    // let transfer_instruction = match spl_token::instruction::transfer_checked(
    //     &spl_token::id(),
    //     &source_pubkey,
    //     &token_pubkey,
    //     &to_pubkey,
    //     &solana_client.pubkey,
    //     &vec![&solana_client.pubkey],
    //     token_transfer_request.amount,
    //     0,
    // )
    let transfer_instruction = match spl_token::instruction::transfer(
        &spl_token::id(),   //Token program id
        &source_account,    //source_pubkey
        &recipient_account, //Destination pubkey
        &solana_client.pubkey,
        &vec![&solana_client.pubkey],
        token_transfer_request.amount, //amount
    ) {
        Ok(instruction) => instruction,
        Err(err) => {
            println!("{:?}", err);
            return Err((
                Status::BadRequest,
                "Failed while creating token transfer instruction".to_string(),
            ));
        }
    };

    let (recent_blockhash, _fee_calculator) = match solana_client.client.get_recent_blockhash() {
        Ok(result) => result,
        Err(err) => {
            println!("{:?}", err);
            return Err((
                Status::BadRequest,
                "Failed to calculate recent blockhash".to_string(),
            ));
        }
    };

    let transfer_transaction = Transaction::new_signed_with_payer(
        &vec![transfer_instruction],
        Some(&solana_client.pubkey),
        &vec![&solana_client.keypair],
        recent_blockhash,
    );

    let tx_signature = match solana_client
        .client
        .send_and_confirm_transaction(&transfer_transaction)
    {
        Ok(signature) => signature.to_string(),
        Err(err) => {
            println!("{:?}", err);
            return Err((
                Status::InternalServerError,
                "Failed to excecute transaction".to_string(),
            ));
        }
    };

    return Ok(Json(TokenTransferResponse {
        token: token_transfer_request.token,
        recipient: token_transfer_request.recipient,
        amount: token_transfer_request.amount,
        transaction: tx_signature,
    }));
}
