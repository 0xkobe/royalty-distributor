use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

pub struct RoyaltyDistributor {
    pub is_initialized: bool, // stored as 1 byte

    // Member's public keys
    pub member_1_pubkey: Pubkey, // 32 bytes
    pub member_2_pubkey: Pubkey, // 32 bytes

    // Shares of members
    pub member_1_shares: u16, // 2 bytes
    pub member_2_shares: u16, // 2 bytes
}

impl IsInitialized for RoyaltyDistributor {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for RoyaltyDistributor {
    const LEN: usize = 69;

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, RoyaltyDistributor::LEN];
        let (is_initialized, member_1_pubkey, member_2_pubkey, member_1_shares, member_2_shares) =
            array_refs![src, 1, 32, 32, 2, 2];

        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(RoyaltyDistributor {
            is_initialized,
            member_1_pubkey: Pubkey::new_from_array(*member_1_pubkey),
            member_2_pubkey: Pubkey::new_from_array(*member_2_pubkey),
            member_1_shares: u16::from_le_bytes(*member_1_shares),
            member_2_shares: u16::from_le_bytes(*member_2_shares),
        })
    }

    // Serialization
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, RoyaltyDistributor::LEN];

        let (
            is_initialized_dst,
            member_1_pubkey_dst,
            member_2_pubkey_dst,
            member_1_shares_dst,
            member_2_shares_dst,
        ) = mut_array_refs![dst, 1, 32, 32, 2, 2];

        let RoyaltyDistributor {
            is_initialized,
            member_1_pubkey,
            member_2_pubkey,
            member_1_shares,
            member_2_shares,
        } = self;

        is_initialized_dst[0] = *is_initialized as u8;
        member_1_pubkey_dst.copy_from_slice(member_1_pubkey.as_ref());
        member_2_pubkey_dst.copy_from_slice(member_2_pubkey.as_ref());
        *member_1_shares_dst = member_1_shares.to_le_bytes();
        *member_2_shares_dst = member_2_shares.to_le_bytes();
    }
}

impl Sealed for RoyaltyDistributor {}
