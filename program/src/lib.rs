pub mod constants;
pub mod errors;
pub mod listings;
pub mod metaplex_cpi;
pub mod offers;
pub mod pda;
pub mod reward_centers;
pub mod state;

use anchor_lang::prelude::*;

use crate::{
    listings::{buy::*, close::*, create::*, update::*},
    offers::{accept::*, close::*, create::*},
    reward_centers::{create::*, edit::*, withdraw::*},
};

declare_id!("RwDDvPp7ta9qqUwxbBfShsNreBaSsKvFcHzMxfBC3Ki");

#[program]
pub mod reward_center {
    use super::*;

    pub fn create_reward_center(
        ctx: Context<CreateRewardCenter>,
        create_reward_center_params: CreateRewardCenterParams,
    ) -> Result<()> {
        reward_centers::create::handler(ctx, create_reward_center_params)
    }

    pub fn edit_reward_center(
        ctx: Context<EditRewardCenter>,
        edit_reward_center_params: EditRewardCenterParams,
    ) -> Result<()> {
        reward_centers::edit::handler(ctx, edit_reward_center_params)
    }

    pub fn withdraw_reward_center_funds(
        ctx: Context<WithdrawRewardCenterFunds>,
        withdraw_reward_center_funds_params: WithdrawRewardCenterFundsParams,
    ) -> Result<()> {
        reward_centers::withdraw::handler(ctx, withdraw_reward_center_funds_params)
    }

    pub fn create_listing(
        ctx: Context<CreateListing>,
        create_listing_params: CreateListingParams,
    ) -> Result<()> {
        listings::create::handler(ctx, create_listing_params)
    }

    pub fn update_listing(
        ctx: Context<UpdateListing>,
        update_listing_params: UpdateListingParams,
    ) -> Result<()> {
        listings::update::handler(ctx, update_listing_params)
    }

    pub fn close_listing(ctx: Context<CloseListing>) -> Result<()> {
        listings::close::handler(ctx)
    }

    pub fn create_offer(
        ctx: Context<CreateOffer>,
        create_offer_params: CreateOfferParams,
    ) -> Result<()> {
        offers::create::handler(ctx, create_offer_params)
    }

    pub fn close_offer(
        ctx: Context<CloseOffer>,
        close_offer_params: CloseOfferParams,
    ) -> Result<()> {
        offers::close::handler(ctx, close_offer_params)
    }

    pub fn buy_listing<'info>(
        ctx: Context<'_, '_, '_, 'info, BuyListing<'info>>,
        buy_listing_params: BuyListingParams,
    ) -> Result<()> {
        listings::buy::handler(ctx, buy_listing_params)
    }

    pub fn accept_offer<'info>(
        ctx: Context<'_, '_, '_, 'info, AcceptOffer<'info>>,
        accept_offer_params: AcceptOfferParams,
    ) -> Result<()> {
        offers::accept::handler(ctx, accept_offer_params)
    }
}
