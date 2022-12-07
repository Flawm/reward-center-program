use std::{path::PathBuf, str::FromStr};

use anchor_lang::AnchorDeserialize;
use anyhow::{Context, Result as AnyhowResult};
use log::info;
use mpl_auction_house::AuctionHouse;
use mpl_auction_house_sdk::{accounts::WithdrawFromTreasuryAccounts, withdraw_from_treasury};
use retry::{delay::Exponential, retry};
use solana_client::rpc_client::RpcClient;
use solana_program::{instruction::Instruction, program_pack::Pack, pubkey::Pubkey};
use solana_sdk::{signer::Signer, transaction::Transaction};
use spl_token::state::Mint;

use crate::config::{parse_keypair, parse_solana_configuration};

/// # Errors
///
/// Will return `Err` if the following happens
/// 1. Auction house fails to parse
/// 2. Withdrawal amount is greater than the treasury balance
pub fn process_withdraw_auction_house_treasury(
    client: &RpcClient,
    keypair_path: &Option<PathBuf>,
    auction_house: &str,
    amount: u64,
) -> AnyhowResult<()> {
    let solana_options = parse_solana_configuration()?;

    let keypair = parse_keypair(keypair_path, &solana_options)?;

    let auction_house_pubkey = Pubkey::from_str(auction_house)
        .context("Failed to parse Pubkey from auction house string")?;

    info!("Getting auction house data");
    let auction_house_data = client
        .get_account_data(&auction_house_pubkey)
        .context("Failed to get auction house data")?;

    let AuctionHouse {
        treasury_withdrawal_destination,
        treasury_mint,
        authority,
        ..
    } = AuctionHouse::deserialize(&mut &auction_house_data[8..])?;

    let token_mint_data = client.get_account_data(&treasury_mint)?;

    let Mint { decimals, .. } = Mint::unpack(&token_mint_data[..])?;

    let amount_to_withdraw_with_decimals =
        amount.saturating_mul(10u64.saturating_pow(decimals.into()));

    let instructions: Vec<Instruction> = vec![withdraw_from_treasury(
        WithdrawFromTreasuryAccounts {
            treasury_mint,
            treasury_withdrawal_destination,
            auction_house: auction_house_pubkey,
            authority,
        },
        amount_to_withdraw_with_decimals,
    )];

    let latest_blockhash = client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&keypair.pubkey()),
        &[&keypair],
        latest_blockhash,
    );

    info!("Withdrawing {} tokens from auction house", amount);

    let tx_hash = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || client.send_and_confirm_transaction(&transaction),
    )?;

    info!("Withdrawal complete. Tx hash {}", tx_hash);

    Ok(())
}
