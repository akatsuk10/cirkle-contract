use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

use crate::error::RwaError;
use crate::state::{UserStake, Vault};


#[derive(Accounts)]
pub struct UnstakeCity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: admin wallet (for PDA seeds)
    pub admin: AccountInfo<'info>,

    pub city_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"protocol_admin", admin.key.as_ref()],
        bump
    )]
    pub admin_vault: Account<'info, Vault>,

    #[account(
        mut,
        constraint = user_city_ata.mint == city_mint.key(),
        constraint = user_city_ata.owner == user.key(),
    )]
    pub user_city_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [
            b"stake",
            user.key().as_ref(),
            city_mint.key().as_ref()
        ],
        bump = user_stake.bump
    )]
    pub user_stake: Account<'info, UserStake>,

    #[account(
        mut,
        associated_token::mint = city_mint,
        associated_token::authority = user_stake,
    )]
    pub stake_vault_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> UnstakeCity<'info> {
    pub fn unstake_city(
        &mut self,
        amount: u64,
        vault_bump: u8,
        city_price_usd: u64,
        sol_price_usd: u64,
    ) -> Result<()> {
        require!(amount > 0, RwaError::InvalidAmount);

        let user_stake = &mut self.user_stake;

        require!(
            user_stake.staked_amount >= amount,
            RwaError::InsufficientStakedAmount
        );

        require!(user_stake.staked_amount > 0, RwaError::NothingStaked);
        require!(city_price_usd > 0, RwaError::InvalidPrice);
        require!(sol_price_usd > 0, RwaError::InvalidPrice);

        let now = Clock::get()?.unix_timestamp;
        let seconds_staked = (now - user_stake.stake_start).max(0) as u64;

        let seconds_per_year: u64 = 31_536_000;

        let city_value_usd = user_stake
            .staked_amount
            .checked_mul(city_price_usd)
            .unwrap();

        let city_value_sol = city_value_usd.checked_div(sol_price_usd).unwrap();

        let reward = city_value_sol
            .checked_mul(seconds_staked)
            .unwrap()
            .checked_mul(6)
            .unwrap()
            .checked_div(100)
            .unwrap()
            .checked_div(seconds_per_year)
            .unwrap();
        if reward > 0 {
            let admin_seeds: &[&[u8]] =
                &[b"protocol_admin", self.admin.key.as_ref(), &[vault_bump]];

            let cpi_reward = Transfer {
                from: self.admin_vault.to_account_info(),
                to: self.user.to_account_info(),
                authority: self.admin_vault.to_account_info(),
            };

            transfer(
                CpiContext::new_with_signer(
                    self.token_program.to_account_info(),
                    cpi_reward,
                    &[admin_seeds],
                ),
                reward,
            )?;
        }

        let binding = self.city_mint.key();
        let stake_seeds: &[&[u8]] = &[
            b"stake",
            self.user.key.as_ref(),
            binding.as_ref(),
            &[user_stake.bump],
        ];

        let cpi_unstake = Transfer {
            from: self.stake_vault_ata.to_account_info(),
            to: self.user_city_ata.to_account_info(),
            authority: user_stake.to_account_info(),
        };

        transfer(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                cpi_unstake,
                &[stake_seeds],
            ),
            amount,
        )?;

        user_stake.staked_amount -= amount;

        if user_stake.staked_amount == 0 {
            user_stake.stake_start = 0;
        }

        Ok(())
    }
}
