use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

use crate::error::RwaError;
use crate::state::{StakeVault, UserStake};

const BASE_RATE: u64 = 1;

#[derive(Accounts)]
pub struct UnstakeCity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: admin wallet (for PDA seeds)
    pub admin: AccountInfo<'info>,

    pub city_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"admin_stake", admin.key.as_ref()],
        bump
    )]
    pub admin_vault: Account<'info, StakeVault>,

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

    #[account(
        mut,
        constraint = user_cirkle_ata.owner == user.key(),
        constraint = user_cirkle_ata.mint == admin_vault.cirkle_mint
    )]
    pub user_cirkle_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = admin_cirkle_vault.mint == admin_vault.cirkle_mint,
        constraint = admin_cirkle_vault.owner == admin_vault.key()
    )]
    pub admin_cirkle_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> UnstakeCity<'info> {
    pub fn unstake_city(&mut self, amount: u64, population: u64,vault_bump:u8) -> Result<()> {
        require!(amount > 0, RwaError::InvalidAmount);

        let user_stake = &mut self.user_stake;

        require!(
            user_stake.staked_amount >= amount,
            RwaError::InsufficientStakedAmount
        );

        let now = Clock::get()?.unix_timestamp;
        let stake_start = user_stake.stake_start;

        let seconds_staked = now - stake_start;
        let days_staked = (seconds_staked / 86400).max(0) as u64;

        let mut multiplier = population / 1_000_000;
        if multiplier < 5 {
            multiplier = 5;
        }
        if multiplier > 10 {
            multiplier = 10;
        }

        let reward = amount
            .checked_mul(days_staked)
            .unwrap()
            .checked_mul(multiplier)
            .unwrap()
            .checked_mul(BASE_RATE)
            .unwrap();

        if reward > 0 {
            let admin_seeds: &[&[u8]] = &[
                b"admin_stake",
                self.admin.key.as_ref(),
                &[vault_bump],
            ];

            let cpi_reward = Transfer {
                from: self.admin_cirkle_vault.to_account_info(),
                to: self.user_cirkle_ata.to_account_info(),
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
