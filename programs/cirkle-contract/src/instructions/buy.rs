use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, MintTo, Token, TokenAccount},
};

use crate::state::CityConfig;
use crate::{error::RwaError, state::Vault};

#[derive(Accounts)]
#[instruction(city_name: String)]
pub struct Buy<'info> {
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
        init_if_needed,
        payer = user,
        space = 8 + CityConfig::INIT_SPACE,
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

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Buy<'info> {
    pub fn buy_token(
        &mut self,
        city_name: String,
        sol_amount: u64,
        circle_rate: u64,
        vault_bump: u8,
    ) -> Result<()> {
        require!(circle_rate > 0, RwaError::RateNotValid);

        let token_amount = sol_amount
            .checked_mul(1_000_000)
            .ok_or(RwaError::Overflow)?
            .checked_div(circle_rate)
            .ok_or(RwaError::DivideByZero)?;

        if self.city_config.mint == Pubkey::default() {
            token::initialize_mint(
                CpiContext::new(
                    self.token_program.to_account_info(),
                    token::InitializeMint {
                        mint: self.mint.to_account_info(),
                        rent: self.system_program.to_account_info(),
                    },
                ),
                6,
                &self.vault.key(),
                Some(&self.vault.key()),
            )?;

            self.city_config.mint = self.mint.key();
            self.city_config.city_name = city_name.clone();
        }

        if self.user_ata.amount == 0 && self.user_ata.owner != self.user.key() {
            let cpi_ctx = CpiContext::new(
                self.associated_token_program.to_account_info(),
                associated_token::Create {
                    payer: self.user.to_account_info(),
                    associated_token: self.user_ata.to_account_info(),
                    authority: self.user.to_account_info(),
                    mint: self.mint.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                    token_program: self.token_program.to_account_info(),
                },
            );
            associated_token::create(cpi_ctx)?;
        }

        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &self.user.key(),
            &self.vault.key(),
            sol_amount,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[self.user.to_account_info(), self.vault.to_account_info()],
        )?;

        self.vault.balance = self
            .vault
            .balance
            .checked_add(sol_amount)
            .ok_or(RwaError::Overflow)?;

        let cpi_accounts = MintTo {
            mint: self.mint.to_account_info(),
            to: self.user_ata.to_account_info(),
            authority: self.vault.to_account_info(),
        };

        let seeds: &[&[u8]] = &[b"protocol_admin", self.admin.key.as_ref(), &[vault_bump]];
        let signer_seeds = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );

        token::mint_to(cpi_ctx, token_amount)?;

        Ok(())
    }
}
