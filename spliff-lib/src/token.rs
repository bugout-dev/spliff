use super::{account::has_token_account, errors::SpliffError, state::SolanaClient};
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

pub struct Token {
    pub address: Pubkey,
    pub signer: Keypair,
    pub supply: f64,
    pub decimals: u8,
    pub mint_tx: String,
    pub minter_token_account: Pubkey,
}

pub fn get_rent_exempt_fee(solana_client: &SolanaClient) -> Result<u64, SpliffError> {
    return match solana_client
        .client
        .get_minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN)
    {
        Ok(fee) => Ok(fee),
        Err(err) => Err(SpliffError::SolanaAPIError(format!(
            "Failed while calculating minimum balance for rent exemption blockhash :\n{:?}",
            err
        ))),
    };
}

pub fn create_token(
    token_supply: &TokenSupply,
    solana_client: &SolanaClient,
) -> Result<Token, SpliffError> {
    let (token_signer, token) = new_throwaway_signer();
    let rent_exempt_fee = match get_rent_exempt_fee(solana_client) {
        Ok(fee) => fee,
        Err(err) => return Err(err),
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
            return Err(SpliffError::SolanaProgramError(format!(
                "Failied to make initialize mint instruction:\n{:?}",
                err
            )));
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
            return Err(SpliffError::SolanaProgramError(format!(
                "Failed to make mint supply instructions:\n{:?}",
                err
            )));
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
            return Err(SpliffError::SolanaAPIError(format!(
                "Failed to calculate recent blockhash:\n{:?}",
                err
            )));
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
            return Err(SpliffError::SolanaAPIError(format!(
                "Failed while excecuting transaction:\n{:?}",
                err
            )));
        }
    };
    let result = Token {
        address: token,
        signer: token_signer,
        supply: token_supply.supply,
        decimals: token_supply.decimals,
        mint_tx: tx_signature,
        minter_token_account: token_account,
    };
    return Ok(result);
}

pub struct TokenTransfer {
    pub sender: Pubkey,
    pub recipient: Pubkey,
    pub token: Pubkey,
    pub sender_account: Pubkey,
    pub recipient_account: Pubkey,
    pub tx_signature: String,
}

pub fn transfer_token(
    token_pubkey: &Pubkey,
    sender: &Keypair,
    recipient: &Pubkey,
    solana_client: &SolanaClient,
    amount: u64,
) -> Result<TokenTransfer, SpliffError> {
    let source_account = get_associated_token_address(&sender.pubkey(), &token_pubkey);
    match has_token_account(&recipient, &token_pubkey, &solana_client) {
        Ok(res) => {
            if !res {
                return Err(SpliffError::SolanaProgramError(format!(
                    "For {} address, token account for {} token not found",
                    &recipient, &token_pubkey
                )));
            }
        }
        Err(err) => {
            return Err(SpliffError::SolanaProgramError(format!(
                "Failed to get account details of {} address for {} token:\n{:?}",
                &recipient, &token_pubkey, err
            )));
        }
    }
    let recipient_account = get_associated_token_address(&recipient, &token_pubkey);
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
        amount, //amount
    ) {
        Ok(instruction) => instruction,
        Err(err) => {
            return Err(SpliffError::SolanaProgramError(format!(
                "Failed while creating token transfer instruction:\n{:?}",
                err
            )));
        }
    };

    let (recent_blockhash, _fee_calculator) = match solana_client.client.get_recent_blockhash() {
        Ok(result) => result,
        Err(err) => {
            return Err(SpliffError::SolanaAPIError(format!(
                "Failed to calculate recent blockhash:\n{:?}",
                err
            )));
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
            return Err(SpliffError::SolanaAPIError(format!(
                "Failed to excecute transaction:\n{:?}",
                err
            )));
        }
    };

    let token_transfer = TokenTransfer {
        sender: sender.pubkey(),
        recipient: recipient.clone(),
        token: token_pubkey.clone(),
        sender_account: source_account,
        recipient_account: recipient_account,
        tx_signature,
    };

    return Ok(token_transfer);
}
