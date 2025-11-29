use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct UserStake {
    pub owner: Pubkey,
    pub city_mint: Pubkey,
    pub staked_amount: u64,
    pub stake_start: i64,
    pub vault_ata: Pubkey, 
    pub bump: u8,
}
