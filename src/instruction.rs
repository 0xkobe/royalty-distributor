// use arrayref::{array_ref, array_refs};
use solana_program::program_error::ProgramError;

use crate::error::RoyaltyDistributorError::InvalidInstruction;

pub enum RoyaltyDistributorInstruction {
    /// Initializes the royalty distributor by:
    /// * Creating and populating a royalty distributor state account
    /// * Transferring ownership of the shared account to the PDA
    ///
    /// Accounts expected:
    /// 0. `[signer]`
    ///    * The account of the initializer
    ///    * Transfering ownership of shared account requires signature of initializer
    ///
    /// 1. `[writable]`
    ///    * Shared account: token account that holds tokens to be shared between members
    ///    * Should be created prior to this instruction and owned by the initializer
    ///    * Should be writable because its ownership will be transfered to the PDA
    ///
    /// 2. `[writable]`
    ///    * State account
    ///    * Stores data about the royalty distributor: member public keys, member shares
    ///
    /// 3. `[]` The rent sysvar
    ///
    /// 4. `[]` The token program account

    /// NOTES: This is a proof of concept that supports only 2 members
    ///
    InitRoyaltyDistributor {
        member_1_shares: u16,
        member_2_shares: u16,
        member_3_shares: u16,
        member_4_shares: u16,
        member_5_shares: u16,
        member_6_shares: u16,
        member_7_shares: u16,
        member_8_shares: u16,
    },

    /// Withdraw instruction
    /// Allow members to withdraw their shares from the shared account
    ///
    /// Accounts expected:
    /// 0. `[signer]`
    ///    * Account of the member executing the withdraw
    ///
    /// 1. `[writable]`
    ///    * State account
    ///    * Stores data about the royalty distributor: member public keys, member shares
    ///
    /// 2. `[writable]`
    ///    * Shared account: token account that holds tokens to be shared between members
    ///
    /// 3. `[]` The token program account
    ///
    /// 4. `[]` The PDA account
    Withdraw {},
}

impl RoyaltyDistributorInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, _) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => Self::unpack_royalty_distributor(),
            1 => Self::Withdraw {},
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_royalty_distributor() -> Self {
        Self::InitRoyaltyDistributor {
            member_1_shares: 3800,
            member_2_shares: 2000,
            member_3_shares: 1000,
            member_4_shares: 700,
            member_5_shares: 700,
            member_6_shares: 600,
            member_7_shares: 600,
            member_8_shares: 600,
        }
    }
}
