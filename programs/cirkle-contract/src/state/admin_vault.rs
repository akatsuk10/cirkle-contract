use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct StakeVault {
    pub admin: Pubkey,
    pub cirkle_mint: Pubkey,
    pub cirkle_vault: Pubkey,
    pub sol_vault: Pubkey,
    pub bump:u8
}
