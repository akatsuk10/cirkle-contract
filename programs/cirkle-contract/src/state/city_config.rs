use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct CityConfig {
    #[max_len(32)]
    pub city_name: String,
    pub mint: Pubkey,
    pub total_supply: u64,
    pub bump: u8,
}
