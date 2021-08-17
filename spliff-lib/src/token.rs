use super::state::SolanaClient;
use serde::Deserialize;

use solana_sdk::{
    program_pack::Pack,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};
use spl_associated_token_account::*;
use spl_token::{
    self,
    instruction::{initialize_mint, mint_to_checked},
    state::Mint,
};

fn new_throwaway_signer() -> (Keypair, Pubkey) {
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey();
    (keypair, pubkey)
}

#[derive(Deserialize, Debug)]
pub struct TokenSupply {
    pub supply: f64,
    pub decimals: u8,
}

// pub fn create_token(
//     token_supply: &TokenSupply,
//     solana_client: &SolanaClient,
// ) -> Result<String, String> {
//     let (token_signer, token) = new_throwaway_signer();
//     let rent_exempt_fee = match solana_client
//         .client
//         .get_minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN)
//     {
//         Ok(fee) => fee,
//         Err(err) => {
//             println!("{:?}", err);
//             return Err(
//                 "Failed while calculating minimum balance for rent exemption blockhash".to_string(),
//             );
//         }
//     };

//     let create_token_instruction = solana_sdk::system_instruction::create_account(
//         &solana_client.pubkey, //fee payer
//         &token,
//         rent_exempt_fee,
//         Mint::LEN as u64,
//         &spl_token::id(), //owner
//     );

//     let initialize_mint_instruction = match initialize_mint(
//         &spl_token::id(),
//         &token,
//         &solana_client.pubkey,
//         None,
//         token_supply.decimals,
//     ) {
//         Ok(instruction) => instruction,
//         Err(err) => {
//             println!("{:?}", err);
//             return Err("Failied to make initialize mint instruction".to_string());
//         }
//     };

//     let create_token_account_instruction = create_associated_token_account(
//         &solana_client.pubkey, //Funding address
//         &solana_client.pubkey, //Wallet address
//         &token,
//     );

//     let token_account = get_associated_token_address(&solana_client.pubkey, &token);
//     let mint_amount = spl_token::ui_amount_to_amount(token_supply.supply, token_supply.decimals);

//     let mint_supply_instruction = match mint_to_checked(
//         &spl_token::id(),
//         &token,
//         &token_account,
//         &solana_client.pubkey,
//         &vec![&solana_client.pubkey, &token],
//         mint_amount,
//         token_supply.decimals,
//     ) {
//         Ok(res) => res,
//         Err(err) => {
//             println!("{:?}", err);
//             return Err("Failed to make mint supply instructions".to_string());
//         }
//     };

//     let instructions = vec![
//         create_token_instruction,
//         initialize_mint_instruction,
//         create_token_account_instruction,
//         mint_supply_instruction,
//     ];

//     let (recent_blockhash, _fee_calculator) = match solana_client.client.get_recent_blockhash() {
//         Ok(result) => result,
//         Err(err) => {
//             println!("{:?}", err);
//             return Err("Failed to calculate recent blockhash".to_string());
//         }
//     };

//     let init_token_transaction = Transaction::new_signed_with_payer(
//         &instructions,
//         Some(&solana_client.pubkey),
//         &vec![&solana_client.keypair, &token_signer],
//         recent_blockhash,
//     );

//     let tx_signature = match solana_client
//         .client
//         .send_and_confirm_transaction(&init_token_transaction)
//     {
//         Ok(signature) => signature.to_string(),
//         Err(err) => {
//             println!("{:?}", err);
//             return Err("Failed while excecuting transaction".to_string());
//         }
//     };
// }
