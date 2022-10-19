use std::{fs::File, path::Path, str::FromStr};

use anyhow::{anyhow, Result as AnyhowResult};
use hpl_reward_center::pda::find_reward_center_address;
use hpl_reward_center_sdk::create_reward_center;
use log::{info, warn};
use mpl_auction_house::pda::find_auction_house_address;
use mpl_auction_house_sdk::{
    create_auction_house, CreateAuctionHouseAccounts, CreateAuctionHouseData,
};
use retry::{delay::Exponential, retry};
use solana_client::rpc_client::RpcClient;
use solana_program::{instruction::Instruction, program_pack::Pack, pubkey::Pubkey};
use solana_sdk::{
    signature::Keypair, signer::Signer, system_instruction::create_account,
    transaction::Transaction,
};
use spl_token::{instruction::initialize_mint, native_mint::id as NATIVE_MINT_ID, state::Mint};

use crate::{
    config::{parse_keypair, parse_solana_config},
    schema::{CreateRewardCenterParams, PayoutOperation},
};

pub fn process_create_reward_center(
    client: RpcClient,
    keypair_path: Option<String>,
    config_file: String,
    auction_house: Option<String>,
    mint_rewards: Option<String>,
) -> AnyhowResult<()> {
    let solana_options = parse_solana_config();
    let keypair = parse_keypair(&keypair_path, &solana_options);
    let wsol_mint = NATIVE_MINT_ID();

    let mut instructions: Vec<Instruction> = vec![];

    let token_program = spl_token::id();

    let auction_house_pubkey = match &auction_house {
        Some(cli_auction_house) => match Pubkey::from_str(&cli_auction_house) {
            Ok(pubkey) => pubkey,
            Err(_) => return Err(anyhow!("Failed to parse Pubkey from mint rewards string")),
        },
        None => find_auction_house_address(&keypair.pubkey(), &wsol_mint).0,
    };

    if auction_house.is_none() {
        info!("Auction house account not passed. Creating a new auction house with default parameters");

        let create_auction_house_accounts = CreateAuctionHouseAccounts {
            treasury_mint: wsol_mint,
            payer: keypair.pubkey(),
            authority: keypair.pubkey(),
            fee_withdrawal_destination: keypair.pubkey(),
            treasury_withdrawal_destination: keypair.pubkey(),
            treasury_withdrawal_destination_owner: keypair.pubkey(),
        };

        let create_auction_house_data = CreateAuctionHouseData {
            seller_fee_basis_points: 100,
            requires_sign_off: false,
            can_change_sale_price: false,
        };

        let create_auction_house_ix =
            create_auction_house(create_auction_house_accounts, create_auction_house_data);

        instructions.push(create_auction_house_ix);
    }

    let reward_mint_keypair = Keypair::new();
    let rewards_mint_pubkey = match &mint_rewards {
        Some(rewards_mint) => match Pubkey::from_str(&rewards_mint) {
            Ok(pubkey) => pubkey,
            Err(_) => return Err(anyhow!("Failed to parse Pubkey from auction house string")),
        },
        None => reward_mint_keypair.pubkey(),
    };

    if mint_rewards.is_none() {
        info!("Rewards mint address not found. Creating a new mint.");
        let rewards_mint_authority_pubkey = keypair.pubkey();

        // Assign account and rent
        let mint_account_rent = client.get_minimum_balance_for_rent_exemption(Mint::LEN)?;

        let allocate_rewards_mint_space_ix = create_account(
            &rewards_mint_authority_pubkey,
            &rewards_mint_pubkey,
            mint_account_rent,
            Mint::LEN as u64,
            &token_program,
        );

        // Initialize rewards mint
        let init_rewards_reward_mint_ix = initialize_mint(
            &token_program,
            &rewards_mint_pubkey,
            &rewards_mint_authority_pubkey,
            Some(&rewards_mint_authority_pubkey),
            9,
        )?;

        instructions.push(allocate_rewards_mint_space_ix);
        instructions.push(init_rewards_reward_mint_ix);
    }

    let CreateRewardCenterParams {
        mathematical_operand,
        payout_numeral,
        seller_reward_payout_basis_points,
    }: CreateRewardCenterParams = match Path::new(&config_file).exists() {
        false => {
            warn!("Create reward center config doesn't exist");
            CreateRewardCenterParams {
                mathematical_operand: PayoutOperation::Divide,
                payout_numeral: 5,
                seller_reward_payout_basis_points: 1000,
            }
        }
        true => {
            let create_reward_center_config_file = File::open(config_file)?;
            serde_json::from_reader(create_reward_center_config_file)?
        }
    };

    let (reward_center_pubkey, _) = find_reward_center_address(&auction_house_pubkey);

    let create_reward_center_ix = create_reward_center(
        keypair.pubkey(),
        rewards_mint_pubkey,
        auction_house_pubkey,
        hpl_reward_center::reward_centers::create::CreateRewardCenterParams {
            reward_rules: {
                hpl_reward_center::state::RewardRules {
                    seller_reward_payout_basis_points,
                    mathematical_operand: match mathematical_operand {
                        PayoutOperation::Divide => {
                            hpl_reward_center::state::PayoutOperation::Divide
                        }
                        PayoutOperation::Multiple => {
                            hpl_reward_center::state::PayoutOperation::Multiple
                        }
                    },
                    payout_numeral,
                }
            },
        },
    );

    instructions.push(create_reward_center_ix);

    let latest_blockhash = client.get_latest_blockhash()?;

    let transaction = match mint_rewards.is_some() {
        true => Transaction::new_signed_with_payer(
            &instructions,
            Some(&keypair.pubkey()),
            &[&keypair],
            latest_blockhash,
        ),
        false => Transaction::new_signed_with_payer(
            &instructions,
            Some(&keypair.pubkey()),
            &[&keypair, &reward_mint_keypair],
            latest_blockhash,
        ),
    };

    let tx_hash = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || client.send_and_confirm_transaction(&transaction),
    )?;

    println!(
        "Reward center address: {}",
        reward_center_pubkey.to_string()
    );

    if mint_rewards.is_none() {
        println!("Rewards mint address: {}", rewards_mint_pubkey.to_string());
    }

    println!("Created in tx: {:?}", &tx_hash);

    Ok(())
}
