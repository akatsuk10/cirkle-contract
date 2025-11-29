use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

use crate::error::RwaError;
use crate::state::UserStake;

#[derive(Accounts)]
pub struct StakeCity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub city_mint: Account<'info, Mint>,

    #[account(
        mut,
        constraint = user_city_ata.mint == city_mint.key(),
        constraint = user_city_ata.owner == user.key(),
    )]
    pub user_city_ata: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        space = UserStake::INIT_SPACE,
        seeds = [
            b"stake",
            user.key.as_ref(),
            city_mint.key().as_ref()
        ],
        bump
    )]
    pub user_stake: Account<'info, UserStake>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = city_mint,
        associated_token::authority = user_stake,
    )]
    pub stake_vault_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> StakeCity<'info> {
    pub fn stake_city(&mut self, amount: u64, stake_bump: u8) -> Result<()> {
        require!(amount > 0, RwaError::InvalidAmount);

        let user = &self.user;
        let user_stake = &mut self.user_stake;

        if user_stake.owner == Pubkey::default() {
            user_stake.owner = user.key();
            user_stake.city_mint = self.city_mint.key();
            user_stake.staked_amount = 0;
            user_stake.stake_start = 0;
            user_stake.vault_ata = self.stake_vault_ata.key();
            user_stake.bump = stake_bump;
        }

        let cpi_accounts = Transfer {
            from: self.user_city_ata.to_account_info(),
            to: self.stake_vault_ata.to_account_info(),
            authority: user.to_account_info(),
        };

        let cpi_program = self.token_program.to_account_info();

        transfer(CpiContext::new(cpi_program, cpi_accounts), amount)?;

        if user_stake.staked_amount == 0 {
            user_stake.stake_start = Clock::get()?.unix_timestamp;
        }

        user_stake.staked_amount = user_stake.staked_amount.checked_add(amount).unwrap();

        Ok(())
    }
}
