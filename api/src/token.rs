use std::str::FromStr;

use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use spliff_lib::{
    account::{list_tokens, TokenBalance},
    errors::SpliffError,
    state::SolanaClient,
    token::{create_token, transfer_token, TokenSupply},
};

#[rocket::get("/")]
pub fn list_tokens_handler(
    solana_client: &State<SolanaClient>,
) -> Result<Json<Vec<TokenBalance>>, Status> {
    return match list_tokens(&solana_client) {
        Ok(results) => Ok(Json(results)),
        Err(err) => {
            println!("Failed to list tokens: {:?}", err);
            return Err(Status::InternalServerError);
        }
    };
}

#[derive(Serialize, Debug)]
pub struct CreateTokenResponse {
    token: String,
    token_account: String,
    balance: String,
    transaction: String,
}

#[rocket::post("/", data = "<token_supply_json>")]
pub fn create_token_handler(
    token_supply_json: Json<TokenSupply>,
    solana_client: &State<SolanaClient>,
) -> Result<Json<CreateTokenResponse>, (Status, String)> {
    let token_supply = token_supply_json.into_inner();

    let token = match create_token(&token_supply, &solana_client) {
        Ok(token) => token,
        Err(SpliffError::SolanaAPIError(msg)) => return Err((Status::InternalServerError, msg)),
        Err(SpliffError::SolanaProgramError(msg)) => {
            return Err((Status::InternalServerError, msg))
        }
        Err(SpliffError::InputError(msg)) => return Err((Status::InternalServerError, msg)),
    };

    return Ok(Json(CreateTokenResponse {
        token: token.address.to_string(),
        token_account: token.minter_token_account.to_string(),
        balance: token.supply.to_string(),
        transaction: token.mint_tx.to_string(),
    }));
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
pub fn transfer_token_handler(
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

    let token_transfer = match transfer_token(
        &token_pubkey,
        &solana_client.keypair,
        &recipient_pubkey,
        solana_client,
        token_transfer_request.amount,
    ) {
        Ok(res) => res,
        Err(err) => match err {
            SpliffError::SolanaAPIError(msg) => return Err((Status::InternalServerError, msg)),
            SpliffError::SolanaProgramError(msg) => return Err((Status::InternalServerError, msg)),
            SpliffError::InputError(msg) => return Err((Status::InternalServerError, msg)),
        },
    };

    return Ok(Json(TokenTransferResponse {
        token: token_transfer_request.token,
        recipient: token_transfer_request.recipient,
        amount: token_transfer_request.amount,
        transaction: token_transfer.tx_signature,
    }));
}
