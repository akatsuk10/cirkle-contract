use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount};

use crate::state::CityConfig;
use crate::{error::RwaError, state::Vault};

#[derive(Accounts)]
#[instruction(city_name: String)]
pub struct Sell<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: Admin that owns the vault PDA
    #[account(mut)]
    pub admin: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"protocol_admin", admin.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,

    /// City configuration - stores metadata about the city's token
    #[account(
        mut,
        seeds = [b"city-config", city_name.as_bytes()],
        bump,
    )]
    pub city_config: Account<'info, CityConfig>,

    /// City-specific mint - unique for each city
    #[account(
        mut,
        seeds = [b"city-mint", city_name.as_bytes()],
        bump,
    )]
    pub city_mint: Account<'info, Mint>,

    /// User's Associated Token Account for this specific city token
    #[account(
        mut,
        associated_token::mint = city_mint,
        associated_token::authority = user
    )]
    pub user_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Sell<'info> {
    pub fn sell_token(
        &mut self,
        city_name: String,
        token_amount: u64,
        circle_rate: u64,
        sol_price_usd: u64,
        _vault_bump: u8,
    ) -> Result<()> {
        require!(circle_rate > 0, RwaError::RateNotValid);
        require!(sol_price_usd > 0, RwaError::RateNotValid);
        require!(token_amount > 0, RwaError::InvalidAmount);

        // Verify the city config matches the mint
        require!(
            self.city_config.mint == self.city_mint.key(),
            RwaError::InvalidMint
        );

        msg!("🏙️ Selling city token: {}", city_name);
        msg!("   Mint address: {}", self.city_mint.key());
        msg!("   SELL DETAILS:");
        msg!("   User: {}", self.user.key());
        msg!("   City: {}", city_name);
        msg!("   Token amount to burn: {}", token_amount);

        // Calculate SOL to return (reverse of buy calculation)
        // Reverse of: tokens = (usd * 1_000_000) / rate
        // So: usd = tokens * rate / 1_000_000
        // Then: lamports = (usd / sol_price) * 1_000_000_000
        // Combined with precision: lamports = (tokens * rate * 1_000_000_000) / (1_000_000 * sol_price)

        let lamports = token_amount
            .checked_mul(circle_rate)
            .and_then(|v| v.checked_mul(1_000_000_000))
            .and_then(|v| v.checked_div(1_000_000))
            .and_then(|v| v.checked_div(sol_price_usd))
            .ok_or(RwaError::DivideByZero)?;

        require!(lamports > 0, RwaError::InvalidAmount);

        msg!("   Lamports to return: {}", lamports);

        // Check vault has sufficient balance
        require!(self.vault.balance >= lamports, RwaError::InsufficientFunds);

        // Burn tokens from user's ATA
        let cpi_accounts = Burn {
            mint: self.city_mint.to_account_info(),
            from: self.user_ata.to_account_info(),
            authority: self.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);
        token::burn(cpi_ctx, token_amount)?;

        msg!("   Tokens burned successfully!");

        // Update city config total supply
        self.city_config.total_supply = self
            .city_config
            .total_supply
            .checked_sub(token_amount)
            .ok_or(RwaError::Overflow)?;

        msg!("   Total city supply: {}", self.city_config.total_supply);

        // Transfer SOL from vault to user by directly manipulating lamports
        // We can't use system program transfer because vault account has data
        // Instead, subtract from vault and add to user
        let vault_info = self.vault.to_account_info();
        let mut vault_lamps = vault_info.lamports.borrow_mut();
        **vault_lamps -= lamports;
        drop(vault_lamps);

        let user_info = self.user.to_account_info();
        let mut user_lamps = user_info.lamports.borrow_mut();
        **user_lamps += lamports;
        drop(user_lamps);

        // Update vault balance
        self.vault.balance = self
            .vault
            .balance
            .checked_sub(lamports)
            .ok_or(RwaError::Overflow)?;

        msg!("   Vault balance: {} lamports", self.vault.balance);
        msg!("✅ Sell completed successfully!");

        Ok(())
    }
}
