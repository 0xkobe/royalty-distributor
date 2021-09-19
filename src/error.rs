use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum RoyaltyDistributorError {
    #[error("Invalid Instruction")]
    InvalidInstruction,
    #[error("Not Rent Exempt")]
    NotRentExempt,
}

impl From<RoyaltyDistributorError> for ProgramError {
    fn from(err: RoyaltyDistributorError) -> Self {
        ProgramError::Custom(err as u32)
    }
}
