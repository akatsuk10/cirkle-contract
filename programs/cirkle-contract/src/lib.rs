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

    pub fn buy(ctx:Context<Buy>,city_name:String,circle_rate:u64,sol_amount:u64)->Result<()>{
        let vault_bump = ctx.bumps.vault;
        ctx.accounts.buy_token(city_name, sol_amount, circle_rate, vault_bump);
        Ok(())
    }
    pub fn sell(ctx:Context<Sell>,circle_rate:u64,token_amount:u64)->Result<()>{
        let vault_bump = ctx.bumps.vault;
        ctx.accounts.sell_token(circle_rate, token_amount, vault_bump);
        Ok(())
    }
    pub fn withdraw(ctx:Context<Withdraw>,amount:u64)->Result<()>{
        ctx.accounts.withdraw(amount);
        Ok(())
    }

}
