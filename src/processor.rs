use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};

use spl_token::{instruction::AuthorityType::AccountOwner, state::Account as TokenAccount};

use crate::{
    error::RoyaltyDistributorError, instruction::RoyaltyDistributorInstruction,
    state::RoyaltyDistributor,
};

pub struct Processor;

impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = RoyaltyDistributorInstruction::unpack(instruction_data)?;
        match instruction {
            RoyaltyDistributorInstruction::InitRoyaltyDistributor {
                member_1_shares,
                member_2_shares,
            } => {
                msg!("Instruction: Init Royalty Distributor");
                Self::process_init_royalty_distributor(
                    accounts,
                    member_1_shares,
                    member_2_shares,
                    program_id,
                )
            }
            RoyaltyDistributorInstruction::Withdraw {} => {
                msg!("Instruction: Withdraw");
                Self::process_withdraw(accounts, program_id)
            }
        }
    }

    fn process_init_royalty_distributor(
        accounts: &[AccountInfo],
        member_1_shares: u16,
        member_2_shares: u16,
        program_id: &Pubkey,
    ) -> ProgramResult {
        // Accounts iterator
        let account_info_iter = &mut accounts.iter();

        // [Account 0] Initializer account
        let init_acct = next_account_info(account_info_iter)?;
        if !init_acct.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // [Account 1] Shared account
        // Should be internally owned by token program
        let shared_acct = next_account_info(account_info_iter)?;
        if *shared_acct.owner != spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        // [Account 2] State account
        let state_acct = next_account_info(account_info_iter)?;

        // [Account 3] Rent sysvar account
        let rent_acct = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
        if !rent_acct.is_exempt(state_acct.lamports(), state_acct.data_len()) {
            return Err(RoyaltyDistributorError::NotRentExempt.into());
        }

        // Ensure that state account is not initialized yet
        let mut state_acct_data = RoyaltyDistributor::unpack_unchecked(&state_acct.data.borrow())?;
        if state_acct_data.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        // [Account 4] Token program account
        let token_program_acct = next_account_info(account_info_iter)?;

        // [Account 5] Member 1 token account
        let member_1_acct = next_account_info(account_info_iter)?;

        // [Account 6] Member 2 token account
        let member_2_acct = next_account_info(account_info_iter)?;

        // Populate data fields on state account
        state_acct_data.is_initialized = true;
        state_acct_data.member_1_pubkey = *member_1_acct.key;
        state_acct_data.member_2_pubkey = *member_2_acct.key;
        state_acct_data.member_1_shares = member_1_shares;
        state_acct_data.member_2_shares = member_2_shares;

        // Store information state account
        RoyaltyDistributor::pack(state_acct_data, &mut state_acct.data.borrow_mut())?;

        // Get a Program Derived Address (PDA)
        let (pda, _bump_seed) = Pubkey::find_program_address(&[b"royalty_distributor"], program_id);

        // Create the 'change owner' instruction
        let owner_change_ix = spl_token::instruction::set_authority(
            token_program_acct.key, // token program id
            shared_acct.key,        // account whose authority we would like to change
            Some(&pda),             // account that should be the new authority of the account
            AccountOwner,           // type of authority change
            init_acct.key,          // current account owner
            &[&init_acct.key],      // public keys signing the cross program invocation (CPI)
        )?;

        // Cross-Program Invocation (CPI)
        msg!("Calling the token program to transfer shared account ownership ...");
        invoke(
            &owner_change_ix,
            &[
                shared_acct.clone(),
                init_acct.clone(),
                token_program_acct.clone(),
            ],
        )?;

        Ok(())
    }

    fn process_withdraw(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        // Accounts iterator
        let account_info_iter = &mut accounts.iter();

        // [Account 0] Account of the member executing the withdraw
        let init_acct = next_account_info(account_info_iter)?;
        if !init_acct.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // [Account 1] State account
        let state_acct = next_account_info(account_info_iter)?;

        // Extract data from state account
        let state_acct_data = RoyaltyDistributor::unpack(&state_acct.data.borrow())?;

        // [Account 2] Shared account
        let shared_acct = next_account_info(account_info_iter)?;
        let shared_acc_data = TokenAccount::unpack(&shared_acct.data.borrow())?;
        let (pda, bump_seed) = Pubkey::find_program_address(&[b"royalty_distributor"], program_id);

        // [Account 4] Token program account
        let token_program_acct = next_account_info(account_info_iter)?;

        // [Account 5] The PDA account
        let pda_acct = next_account_info(account_info_iter)?;

        // Calculate if the member can withdraw the amount requested
        let shared_acc_balance = shared_acc_data.amount as f64;
        let member_1_shares = state_acct_data.member_1_shares as f64;
        let member_2_shares = state_acct_data.member_2_shares as f64;

        let member1_amount = (shared_acc_balance * member_1_shares / 10000f64) as u64;
        let member2_amount = (shared_acc_balance * member_2_shares / 10000f64) as u64;

        // Withdraw transfer instruction
        let withdraw_transfer_ix1 = spl_token::instruction::transfer(
            token_program_acct.key,           // token program account
            shared_acct.key,                  // source account
            &state_acct_data.member_1_pubkey, // destination account
            &pda,                             // authority account
            &[&pda],                          // signer account
            member1_amount,                   // amount
        )?;

        // Withdraw transfer instruction
        let withdraw_transfer_ix2 = spl_token::instruction::transfer(
            token_program_acct.key,           // token program account
            shared_acct.key,                  // source account
            &state_acct_data.member_2_pubkey, // destination account
            &pda,                             // authority account
            &[&pda],                          // signer account
            member2_amount,                   // amount
        )?;

        // let withdraw_acct = AccountInfo::new(&state_acct_data.member_1_pubkey, false, false, 0, [], owner, false, rent_epoch)

        msg!("Calling the token program to execute the withdraw ...");
        invoke_signed(
            &withdraw_transfer_ix1,
            &[
                shared_acct.clone(),
                // withdraw_acct.clone(),
                pda_acct.clone(),
                token_program_acct.clone(),
            ],
            &[&[&b"royalty_distributor"[..], &[bump_seed]]],
        )?;

        invoke_signed(
            &withdraw_transfer_ix2,
            &[
                shared_acct.clone(),
                // withdraw_acct.clone(),
                pda_acct.clone(),
                token_program_acct.clone(),
            ],
            &[&[&b"royalty_distributor"[..], &[bump_seed]]],
        )?;

        RoyaltyDistributor::pack(state_acct_data, &mut state_acct.data.borrow_mut())?;

        Ok(())
    }
}
