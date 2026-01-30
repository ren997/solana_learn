use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
use crate::error::VaultError;

/// 账户检查 trait
pub trait AccountCheck {
    fn check(account: &AccountInfo) -> Result<(), ProgramError>;
}

/// 签名者账户
pub struct SignerAccount;

impl AccountCheck for SignerAccount {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if !account.is_signer() {
            return Err(VaultError::NotSigner.into());
        }
        Ok(())
    }
}

/// 系统账户
pub struct SystemAccount;

impl AccountCheck for SystemAccount {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if !account.is_owned_by(&pinocchio_system::ID) {
            return Err(VaultError::InvalidOwner.into());
        }
        Ok(())
    }
}

/// 可写账户检查
pub fn check_writable(account: &AccountInfo) -> Result<(), ProgramError> {
    if !account.is_writable() {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}

/// PDA 验证
pub fn verify_pda(
    account: &AccountInfo,
    seeds: &[&[u8]],
    program_id: &Pubkey,
) -> Result<u8, ProgramError> {
    let (expected_key, bump) = Pubkey::find_program_address(seeds, program_id);
    if account.key() != &expected_key {
        return Err(ProgramError::InvalidSeeds);
    }
    Ok(bump)
}
