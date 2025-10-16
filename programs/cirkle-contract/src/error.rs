use anchor_lang::prelude::*;

#[error_code]
pub enum RwaError {
    #[msg("City not found")]
    CityNotFound,
    #[msg("Insufficient funds")]
    InsufficientFunds,
}
