use std::env::var;

use solana_client::rpc_client::RpcClient;
use solana_sdk::signer::keypair::{read_keypair_file, Keypair};

pub struct SolanaClient {
    pub client: RpcClient,
    pub keypair: Keypair,
}

#[derive(Debug)]
pub enum StateError {
    SolanaMissingAPIUrl,
    SolanaMissingKeypairPath,
    SolanaKeypairLoadError,
}

pub fn solana_client_from_env() -> Result<SolanaClient, StateError> {
    let solana_api_url = match var("SPLIFF_SOLANA_API_URL") {
        Ok(api_url) => api_url,
        Err(_) => return Err(StateError::SolanaMissingAPIUrl),
    };
    let solana_keypair_path = match var("SPLIFF_SOLANA_KEYPAIR_PATH") {
        Ok(keypair_path) => keypair_path,
        Err(_) => return Err(StateError::SolanaMissingKeypairPath),
    };

    let rpc_client = RpcClient::new(solana_api_url);

    let solana_keypair = match read_keypair_file(solana_keypair_path) {
        Ok(keypair) => keypair,
        Err(_) => return Err(StateError::SolanaKeypairLoadError),
    };

    return Ok(SolanaClient {
        client: rpc_client,
        keypair: solana_keypair,
    });
}
