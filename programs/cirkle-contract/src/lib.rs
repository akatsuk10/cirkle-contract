#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

declare_id!("4rGhCxGhfqRnmsNzwe9e9VUE6nFvfXuxmep6BUhbcrne");
mod error;
mod instructions;
mod state;

use instructions::*;
#[program]
pub mod cirkle_contract {
    use super::*;

    pub fn vault_initialize(ctx: Context<AdminVault>) -> Result<()> {
        let bump = ctx.bumps.admin_vault;
        ctx.accounts.create_vault(bump)?;
        Ok(())
    }

    pub fn buy(
        ctx: Context<Buy>,
        city_name: String,
        sol_amount: u64,
        circle_rate: u64,
        sol_price_usd: u64,
        metadata_uri: String,
    ) -> Result<()> {
        let vault_bump = ctx.bumps.vault;
        ctx.accounts.buy_token(
            city_name,
            sol_amount,
            circle_rate,
            sol_price_usd,
            vault_bump,
            metadata_uri,
        )?;
        Ok(())
    }
    pub fn sell(
        ctx: Context<Sell>,
        city_name: String,
        circle_rate: u64,
        sol_price_usd: u64,
        token_amount: u64,
    ) -> Result<()> {
        let vault_bump = ctx.bumps.vault;
        ctx.accounts.sell_token(
            city_name,
            token_amount,
            circle_rate,
            sol_price_usd,
            vault_bump,
        )?;
        Ok(())
    }
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)?;
        Ok(())
    }
    pub fn stake(ctx: Context<StakeCity>, amount: u64) -> Result<()> {
        let bump = ctx.bumps.user_stake;
        ctx.accounts.user_stake.bump = bump;
        ctx.accounts.stake_city(amount)?;
        Ok(())
    }
    pub fn unstake(
        ctx: Context<UnstakeCity>,
        amount: u64,
        city_price_usd: u64,
        sol_price_usd: u64,
    ) -> Result<()> {
        let bump = ctx.bumps.admin_vault;
        ctx.accounts
            .unstake_city(amount, bump, city_price_usd, sol_price_usd)?;
        Ok(())
    }
    pub fn claim(ctx: Context<ClaimReward>, city_price_usd: u64, sol_price_usd: u64) -> Result<()> {
        let bump = ctx.bumps.admin_vault;
        ctx.accounts
            .claim_reward(bump, city_price_usd, sol_price_usd)?;
        Ok(())
    }
    pub fn close_stake(ctx: Context<CloseStake>) -> Result<()> {
        ctx.accounts.close_stake()?;
        Ok(())
    }
}
