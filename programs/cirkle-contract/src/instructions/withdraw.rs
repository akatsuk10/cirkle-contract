use anchor_lang::{prelude::*, solana_program::{program::invoke_signed, system_instruction::transfer}};
use crate::state::Vault;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"protocol_admin", admin.key().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, Vault>,

    /// CHECK: receiver of the SOL
    #[account(mut)]
    pub recipient: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        require!(amount > 0, crate::error::RwaError::AmountNotValid);
        require!(self.vault.balance >= amount, crate::error::RwaError::InsufficientFunds);

        let seeds = &[
            b"protocol_admin",
            self.admin.key.as_ref(),
            &[self.vault.bump],
        ];
        let signer = &[&seeds[..]];

        let ix = transfer(
            &self.vault.key(),
            &self.recipient.key(),
            amount,
        );

        invoke_signed(
            &ix,
            &[self.vault.to_account_info(), self.recipient.to_account_info()],
            signer,
        )?;

        self.vault.balance = self
            .vault
            .balance
            .checked_sub(amount)
            .ok_or(crate::error::RwaError::Overflow)?;

        Ok(())
    }
}
