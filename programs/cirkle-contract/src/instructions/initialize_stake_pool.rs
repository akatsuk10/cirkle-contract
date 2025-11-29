use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::state::StakeVault;
#[derive(Accounts)]
pub struct InitializeStakeVault<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    /// CHECK: CIRKLE mint passed by admin
    pub cirkle_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = admin,
        space = StakeVault::INIT_SPACE,
        seeds = [b"admin_stake",admin.key.as_ref()],
        bump
    )]
    pub admin_vault: Account<'info, StakeVault>,

    #[account(
        init,
        payer = admin,
        associated_token::mint = cirkle_mint,
        associated_token::authority = admin_vault,
    )]
    pub cirkle_vault_ata: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> InitializeStakeVault<'info> {
    pub fn initialize_stake_vault(&mut self,bump:u8) -> Result<()> {
        let admin_vault_key = self.admin_vault.key();
        let vault = &mut self.admin_vault;

        vault.admin = self.admin.key();
        vault.cirkle_mint = self.cirkle_mint.key();
        vault.cirkle_vault = self.cirkle_vault_ata.key();
        vault.sol_vault = admin_vault_key;
        vault.bump = bump;

        Ok(())
    }
}
