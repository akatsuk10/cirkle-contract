use anchor_lang::prelude::*;

use crate::state::Vault;

#[derive(Accounts)]
pub struct AdminVault<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init_if_needed,
        payer= admin,
        space = Vault::INIT_SPACE + 8,
        seeds = [b"protocol_admin",admin.key().as_ref()],
        bump,
    )]
    pub admin_vault: Account<'info, Vault>,

    pub system_program: Program<'info, System>,
}

impl<'info> AdminVault<'info> {
    pub fn create_vault(&mut self, bump: u8) -> Result<()> {
        let vault = &mut self.admin_vault;

        vault.set_inner(Vault {
            authority: *self.admin.key,
            balance: 0,
            bump,
        });

        Ok(())
    }
}
