use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer as SystemTransfer};
use anchor_spl::token::{Mint, Token};

use crate::error::RwaError;
use crate::state::{UserStake, Vault};

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: admin wallet
    pub admin: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"protocol_admin", admin.key().as_ref()],
        bump
    )]
    pub admin_vault: Account<'info, Vault>,

    pub city_mint: Account<'info, Mint>,

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

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> ClaimReward<'info> {
    pub fn claim_reward(
        &mut self,
        vault_bump: u8,
        city_price_usd: u64,
        sol_price_usd: u64,
    ) -> Result<()> {
        let user_stake = &mut self.user_stake;

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

        require!(reward > 0, RwaError::NoRewardsAvailable);

        let admin_seeds: &[&[u8]] = &[b"protocol_admin", self.admin.key.as_ref(), &[vault_bump]];

        let cpi_account = SystemTransfer {
            from: self.admin_vault.to_account_info(),
            to: self.user.to_account_info(),
        };

        let binding = [admin_seeds];
        let cpi_ctx = CpiContext::new_with_signer(
            self.system_program.to_account_info(),
            cpi_account,
            &binding,
        );

        system_program::transfer(cpi_ctx, reward)?;

        user_stake.stake_start = now;

        Ok(())
    }
}
