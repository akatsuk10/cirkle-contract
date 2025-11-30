use anchor_lang::prelude::*;

#[error_code]
pub enum RwaError {
    #[msg("City not found")]
    CityNotFound,
    #[msg("Insufficient funds")]
    InsufficientFunds,
    #[msg("Rate Not Valid")]
    RateNotValid,
    #[msg("Token Overflow")]
    Overflow,
    #[msg("Divide by zero")]
    DivideByZero,
    #[msg("Amount not valid")]
    AmountNotValid,
    #[msg("Invalid Mint")]
    InvalidMint,
    #[msg("Invalid Amount")]
    InvalidAmount,
    #[msg("Insufficient Staked Amount")]
    InsufficientStakedAmount,
    #[msg("Nothing Staked")]
    NothingStaked,
    #[msg("No Rewards Available")]
    NoRewardsAvailable,
    #[msg("Invalid Price")]
    InvalidPrice,
}
