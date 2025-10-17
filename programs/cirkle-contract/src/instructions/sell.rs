use anchor_lang::{prelude::*, solana_program::{program::invoke_signed, system_instruction::transfer}};
use anchor_spl::{
    token::{self, Burn, Mint, Token, TokenAccount, Token as TokenProgram},
};

use crate::state::CityConfig;
use crate::{error::RwaError, state::Vault};

#[derive(Accounts)]
#[instruction(city_name: String)]
pub struct Sell<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"protocol_admin", admin.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        seeds = [b"city-mint", city_name.as_bytes()],
        bump,
    )]
    pub city_config: Account<'info, CityConfig>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user
    )]
    pub user_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, TokenProgram>,
    pub system_program: Program<'info, System>,
}

impl<'info> Sell<'info> {
    pub fn sell_token(
        &mut self,
        circle_rate: u64,
        token_amount: u64,
        vault_bump: u8,
    ) -> Result<()> {
        require!(circle_rate > 0, RwaError::RateNotValid);
        require!(token_amount > 0, RwaError::AmountNotValid);

        require!(self.mint.key() == self.city_config.mint, RwaError::InvalidMint);

        let sol_amount = token_amount
            .checked_mul(circle_rate)
            .ok_or(RwaError::Overflow)?
            .checked_div(1_000_000)
            .ok_or(RwaError::DivideByZero)?;

        require!(self.vault.balance >= sol_amount, RwaError::InsufficientFunds);

        let cpi_accounts = Burn {
            mint: self.mint.to_account_info(),
            from: self.user_ata.to_account_info(),
            authority: self.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);
        token::burn(cpi_ctx, token_amount)?;

        let ix = transfer(
            &self.vault.key(),
            &self.user.key(),
            sol_amount,
        );
        let seeds: &[&[u8]] = &[b"protocol_admin", self.admin.key.as_ref(), &[vault_bump]];
        let signer_seeds = &[&seeds[..]];

        invoke_signed(
            &ix,
            &[self.vault.to_account_info(), self.user.to_account_info()],
            signer_seeds,
        )?;

        self.vault.balance = self
            .vault
            .balance
            .checked_sub(sol_amount)
            .ok_or(RwaError::Overflow)?;

        Ok(())
    }
}
