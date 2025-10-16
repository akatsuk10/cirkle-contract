use anchor_lang::prelude::*;

declare_id!("5s8sxuXaNoJp1imQ6uD89jGefKXXL1jgcNE7xUYFPgrs");
mod instructions;
mod state;
mod error;

use instructions::*;
#[program]
pub mod cirkle_contract {

    use super::*;

    pub fn vault_initialize(ctx:Context<AdminVault>)->Result<()>{
        let bump = ctx.bumps.admin_vault;
        ctx.accounts.create_vault(bump);
        Ok(())
    }

}
