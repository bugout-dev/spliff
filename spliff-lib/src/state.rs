use std::env::var;

use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::{
    keypair::{read_keypair_file, Keypair},
    Signer,
};

#[derive(Debug)]
pub enum StateError {
    SolanaMissingAPIUrl(&'static str),
    SolanaMissingKeypairPath(&'static str),
    SolanaKeypairLoadError(&'static str),
}

pub struct SolanaClient {
    pub client: RpcClient,
    pub keypair: Keypair,
    pub pubkey: Pubkey,
}

impl SolanaClient {
    pub fn from_env() -> Result<SolanaClient, StateError> {
        let solana_api_url = match var("SPLIFF_SOLANA_API_URL") {
            Ok(api_url) => api_url,
            Err(_) => {
                return Err(StateError::SolanaMissingAPIUrl(
                    "SPLIFF_SOLANA_API_URL environment variable not set",
                ))
            }
        };

        let rpc_client = RpcClient::new(solana_api_url);

        let solana_keypair_path = match var("SPLIFF_SOLANA_KEYPAIR_PATH") {
            Ok(keypair_path) => keypair_path,
            Err(_) => {
                return Err(StateError::SolanaMissingKeypairPath(
                    "SPLIFF_SOLANA_KEYPAIR_PATH environment variable not set",
                ))
            }
        };

        let solana_keypair = match read_keypair_file(solana_keypair_path) {
            Ok(keypair) => keypair,
            Err(_) => {
                return Err(StateError::SolanaKeypairLoadError(
                    "Failed to load keypair from path SPLIFF_SOLANA_KEYPAIR_PATH",
                ));
            }
        };

        let solana_pubkey = solana_keypair.pubkey();

        return Ok(SolanaClient {
            client: rpc_client,
            keypair: solana_keypair,
            pubkey: solana_pubkey,
        });
    }
}
