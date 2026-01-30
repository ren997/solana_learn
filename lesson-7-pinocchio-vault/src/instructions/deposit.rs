use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};
use pinocchio_system::instructions::Transfer;

use crate::{
    error::VaultError,
    helpers::{check_writable, verify_pda, AccountCheck, SignerAccount},
    state::Vault,
};

/// 存款指令的账户
pub struct DepositAccounts<'a> {
    /// 存款人(签名者)
    pub owner: &'a AccountInfo,
    /// 金库 PDA 账户
    pub vault: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for DepositAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [owner, vault, _system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // 验证 owner 是签名者
        SignerAccount::check(owner)?;

        // 验证 vault 可写
        check_writable(vault)?;

        Ok(Self { owner, vault })
    }
}

/// 存款指令数据
pub struct DepositInstructionData {
    pub amount: u64,
}

impl TryFrom<&[u8]> for DepositInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // 验证数据长度(8 字节 u64)
        if data.len() != 8 {
            return Err(ProgramError::InvalidInstructionData);
        }

        // 从小端字节转换为 u64
        let amount = u64::from_le_bytes(data.try_into().unwrap());

        // 验证金额大于零
        if amount == 0 {
            return Err(VaultError::InvalidAmount.into());
        }

        Ok(Self { amount })
    }
}

/// 存款指令
pub struct Deposit<'a> {
    pub accounts: DepositAccounts<'a>,
    pub data: DepositInstructionData,
}

impl<'a> Deposit<'a> {
    /// 指令判别器
    pub const DISCRIMINATOR: u8 = 1;

    /// 从账户和数据创建指令
    pub fn try_from(
        data: &[u8],
        accounts: &'a [AccountInfo],
    ) -> Result<Self, ProgramError> {
        let accounts = DepositAccounts::try_from(accounts)?;
        let data = DepositInstructionData::try_from(data)?;
        Ok(Self { accounts, data })
    }

    /// 执行存款逻辑
    pub fn process(&self, program_id: &Pubkey) -> ProgramResult {
        // 1. 验证 vault 是正确的 PDA
        let seeds = &[b"vault", self.accounts.owner.key().as_ref()];
        verify_pda(self.accounts.vault, seeds, program_id)?;

        // 2. 验证 vault 已初始化且所有者正确
        let vault_data = self.accounts.vault.try_borrow_data()?;
        let vault = Vault::from_bytes(&vault_data)?;
        vault.check_initialized()?;
        vault.check_owner(self.accounts.owner.key().as_ref())?;
        drop(vault_data); // 释放借用

        // 3. 执行转账(从 owner 到 vault)
        Transfer {
            from: self.accounts.owner,
            to: self.accounts.vault,
            lamports: self.data.amount,
        }
        .invoke()?;

        #[cfg(not(feature = "perf"))]
        pinocchio::msg!(
            "Deposited {} lamports to vault",
            self.data.amount
        );

        Ok(())
    }
}
