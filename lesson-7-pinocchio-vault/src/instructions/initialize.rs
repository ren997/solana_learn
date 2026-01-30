use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};
use pinocchio_system::{instructions::CreateAccount, system_program};

use crate::{
    helpers::{check_writable, verify_pda, AccountCheck, SignerAccount},
    state::Vault,
};

/// 初始化指令的账户
pub struct InitializeAccounts<'a> {
    /// 金库所有者(签名者,支付者)
    pub owner: &'a AccountInfo,
    /// 金库 PDA 账户
    pub vault: &'a AccountInfo,
    /// 系统程序
    pub system_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for InitializeAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // 解构账户切片
        let [owner, vault, system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // 验证 owner 是签名者
        SignerAccount::check(owner)?;

        // 验证 vault 可写
        check_writable(vault)?;

        // 验证系统程序
        if system_program.key() != &system_program::ID {
            return Err(ProgramError::IncorrectProgramId);
        }

        Ok(Self {
            owner,
            vault,
            system_program,
        })
    }
}

/// 初始化指令
pub struct Initialize<'a> {
    pub accounts: InitializeAccounts<'a>,
}

impl<'a> Initialize<'a> {
    /// 指令判别器
    pub const DISCRIMINATOR: u8 = 0;

    /// 从账户创建指令
    pub fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, ProgramError> {
        let accounts = InitializeAccounts::try_from(accounts)?;
        Ok(Self { accounts })
    }

    /// 执行初始化逻辑
    pub fn process(&self, program_id: &Pubkey) -> ProgramResult {
        // 1. 验证 vault 是正确的 PDA
        let seeds = &[b"vault", self.accounts.owner.key().as_ref()];
        let bump = verify_pda(self.accounts.vault, seeds, program_id)?;

        // 2. 创建 vault 账户
        let rent_lamports = pinocchio_system::rent::Rent::get()?
            .minimum_balance(Vault::LEN);

        CreateAccount {
            from: self.accounts.owner,
            to: self.accounts.vault,
            lamports: rent_lamports,
            space: Vault::LEN as u64,
            owner: program_id,
        }
        .invoke()?;

        // 3. 初始化 vault 数据
        let mut vault_data = self.accounts.vault.try_borrow_mut_data()?;
        let vault = Vault::from_bytes_mut(&mut vault_data)?;
        vault.initialize(self.accounts.owner.key().as_ref(), bump)?;

        #[cfg(not(feature = "perf"))]
        pinocchio::msg!("Vault initialized for owner: {:?}", self.accounts.owner.key());

        Ok(())
    }
}
