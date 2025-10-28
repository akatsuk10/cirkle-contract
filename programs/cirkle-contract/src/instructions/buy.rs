use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{create_metadata_accounts_v3, CreateMetadataAccountsV3},
    token::{self, Mint, MintTo, Token, TokenAccount},
};

use crate::error::RwaError;
use crate::state::{CityConfig, Vault};

#[derive(Accounts)]
#[instruction(city_name: String)]
pub struct Buy<'info> {
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
        init_if_needed,
        payer = user,
        seeds = [b"city-config", city_name.as_bytes()],
        bump,
        space = 8 + CityConfig::INIT_SPACE,
    )]
    pub city_config: Account<'info, CityConfig>,

    /// City-specific mint - unique for each city
    /// This gets created on the FIRST buy for this city
    #[account(
        init_if_needed,
        payer = user,
        seeds = [b"city-mint", city_name.as_bytes()],
        bump,
        mint::decimals = 6,
        mint::authority = vault,
        mint::freeze_authority = vault
    )]
    pub city_mint: Account<'info, Mint>,

    /// User's Associated Token Account for this specific city token
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = city_mint,
        associated_token::authority = user
    )]
    pub user_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

    #[account(mut)]
    /// CHECK: Metaplex will verify this is the correct metadata PDA
    pub metadata: UncheckedAccount<'info>,
    pub token_metadata_program: Program<'info, anchor_spl::metadata::Metadata>,
}

impl<'info> Buy<'info> {
    pub fn buy_token(
        &mut self,
        city_name: String,
        lamports: u64,
        circle_rate: u64,
        sol_price_usd: u64,
        vault_bump: u8,
        metadata_uri: String,
    ) -> Result<()> {
        require!(circle_rate > 0, RwaError::RateNotValid);
        require!(sol_price_usd > 0, RwaError::RateNotValid);
        require!(lamports > 0, RwaError::InvalidAmount);

        if self.city_config.mint == Pubkey::default() {
            self.city_config.mint = self.city_mint.key();
            self.city_config.city_name = city_name.clone();
            self.city_config.total_supply = 0;
            self.city_config.metadata_uri = metadata_uri.clone();
            msg!("NEW CITY TOKEN CREATED: {}", city_name);
            msg!("   Mint address: {}", self.city_mint.key());
            msg!("   Metadata URI: {}", metadata_uri);
        } else {
            msg!("Buying existing city token: {}", city_name);
            msg!("   Mint address: {}", self.city_mint.key());
        }

        // Multiply first, then divide to preserve precision
        let sol_amount_usd = lamports
            .checked_mul(sol_price_usd)
            .and_then(|v| v.checked_div(1_000_000_000))
            .ok_or(RwaError::DivideByZero)?;

        // Apply decimals BEFORE dividing to preserve precision
        // tokens = (USD value * 1_000_000) / rate
        let token_amount_with_decimals = sol_amount_usd
            .checked_mul(1_000_000)
            .and_then(|v| v.checked_div(circle_rate))
            .ok_or(RwaError::DivideByZero)?;

        // Calculate sol units for logging
        let sol_units = lamports
            .checked_div(1_000_000_000)
            .unwrap_or(0);

        msg!("   PURCHASE DETAILS:");
        msg!("   User: {}", self.user.key());
        msg!("   City: {}", city_name);
        msg!("   Lamports paid: {}", lamports);
        msg!("   SOL amount: {}.{}", sol_units, lamports % 1_000_000_000);
        msg!("   USD value: ${}", sol_amount_usd);
        msg!("   Rate per token: ${}", circle_rate);
        msg!(
            "   Tokens to mint: {} (with decimals)",
            token_amount_with_decimals
        );

        anchor_lang::solana_program::program::invoke(
            &anchor_lang::solana_program::system_instruction::transfer(
                &self.user.key(),
                &self.vault.key(),
                lamports,
            ),
            &[
                self.user.to_account_info(),
                self.vault.to_account_info(),
                self.system_program.to_account_info(),
            ],
        )?;

        self.vault.balance = self
            .vault
            .balance
            .checked_add(lamports)
            .ok_or(RwaError::Overflow)?;

        self.city_config.total_supply = self
            .city_config
            .total_supply
            .checked_add(token_amount_with_decimals)
            .ok_or(RwaError::Overflow)?;

        msg!("   Vault balance: {} lamports", self.vault.balance);
        msg!("   Total city supply: {}", self.city_config.total_supply);

        let binding = self.admin.key();
        let signer_seeds: &[&[u8]] = &[b"protocol_admin", binding.as_ref(), &[vault_bump]];
        let signer = &[&signer_seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.city_mint.to_account_info(),
                to: self.user_ata.to_account_info(),
                authority: self.vault.to_account_info(),
            },
            signer,
        );
        token::mint_to(cpi_ctx, token_amount_with_decimals)?;

        msg!(" Tokens minted successfully!");

        if self.city_config.mint == self.city_mint.key() {
            msg!("Creating Metaplex metadata for token...");

            let symbol = city_name.chars().take(10).collect::<String>();

            let cpi_program = self.token_metadata_program.to_account_info();
            let cpi_accounts = CreateMetadataAccountsV3 {
                metadata: self.metadata.to_account_info(),
                mint: self.city_mint.to_account_info(),
                mint_authority: self.vault.to_account_info(),
                update_authority: self.vault.to_account_info(),
                payer: self.user.to_account_info(),
                system_program: self.system_program.to_account_info(),
                rent: self.rent.to_account_info(),
            };

            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

            let data = mpl_token_metadata::types::DataV2 {
                name: city_name.clone(),
                symbol,
                uri: metadata_uri.clone(),
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            };

            create_metadata_accounts_v3(
                cpi_ctx,
                data,
                false,
                true,
                None,
            )?;

            msg!("Metadata created successfully!");
        }

        Ok(())
    }
}
