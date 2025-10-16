use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{transfer, Mint, Token, TokenAccount, Transfer}};

use crate::state::CityConfig;

#[derive(Accounts)]
#[instruction(city_name:String)]
pub struct Buy<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: Vault PDA stores all collected SOL
    #[account(mut)]
    pub vault: AccountInfo<'info>,

    #[account(
        seeds = [b"city-mint", city_name.as_bytes()],
        bump,
        init_if_needed,
        payer = user,
        space = 8 + CityConfig::INIT_SPACE,
    )]
    pub city_config: Account<'info, CityConfig>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub user_ata: Account<'info, TokenAccount>,

    /// CHECK: Mint authority PDA
    pub mint_authority: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}