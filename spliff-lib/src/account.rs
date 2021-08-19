use super::state::SolanaClient;
use serde::Serialize;
use solana_account_decoder::{parse_token::TokenAccountType, UiAccountData};
use solana_client::client_error::ClientError;
use solana_client::{rpc_request::TokenAccountsFilter, rpc_response::RpcKeyedAccount};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::system_program;
use spl_associated_token_account::get_associated_token_address;
#[derive(Serialize, Debug)]
pub struct TokenBalance {
    token_address: String,
    token_account: String,
    balance: String,
    owner_pubkey: String,
}

pub fn list_tokens(solana_client: &SolanaClient) -> Result<Vec<TokenBalance>, ClientError> {
    let accounts: Vec<TokenBalance> = solana_client
        .client
        .get_token_accounts_by_owner(
            &solana_client.pubkey,
            TokenAccountsFilter::ProgramId(spl_token::id()),
        )?
        .iter()
        .map(|token_account| -> TokenBalance {
            parse_account(token_account, &solana_client.pubkey)
        })
        .collect();
    return Ok(accounts);
}

pub fn parse_account(account: &RpcKeyedAccount, owner: &Pubkey) -> TokenBalance {
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

pub fn has_token_account(
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
