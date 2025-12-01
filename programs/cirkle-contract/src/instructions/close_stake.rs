use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};

#[derive(Accounts)]
pub struct CloseStake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    ///CHECK:city mint
    pub city_mint: UncheckedAccount<'info>,

    ///CHECK:USER STAKE
    #[account(
        mut,
        seeds = [
            b"stake",
            user.key().as_ref(),
            city_mint.key().as_ref()
        ],
        bump
    )]
    pub user_stake: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> CloseStake<'info> {
    pub fn close_stake(&self) -> Result<()> {
        let lamports = self.user_stake.lamports();

        let cpi_accounts = Transfer {
            from: self.user_stake.to_account_info(),
            to: self.user.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            self.system_program.to_account_info(),
            cpi_accounts,
        );

        system_program::transfer(cpi_ctx, lamports)?;

        Ok(())
    }
}
