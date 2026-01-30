use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};
use pinocchio_system::instructions::transfer;

use crate::{
    error::VaultError,
    helpers::{check_writable, verify_pda, AccountCheck, SignerAccount},
    state::Vault,
};

/// 取款指令的账户
pub struct WithdrawAccounts<'a> {
    /// 取款人(签名者,必须是 vault 所有者)
    pub owner: &'a AccountInfo,
    /// 金库 PDA 账户
    pub vault: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for WithdrawAccounts<'a> {
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

/// 取款指令数据
pub struct WithdrawInstructionData {
    pub amount: u64,
}

impl TryFrom<&[u8]> for WithdrawInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() != 8 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let amount = u64::from_le_bytes(data.try_into().unwrap());

        if amount == 0 {
            return Err(VaultError::InvalidAmount.into());
        }

        Ok(Self { amount })
    }
}

/// 取款指令
pub struct Withdraw<'a> {
    pub accounts: WithdrawAccounts<'a>,
    pub data: WithdrawInstructionData,
}

impl<'a> Withdraw<'a> {
    /// 指令判别器
    pub const DISCRIMINATOR: u8 = 2;

    /// 从账户和数据创建指令
    pub fn try_from(
        data: &[u8],
        accounts: &'a [AccountInfo],
    ) -> Result<Self, ProgramError> {
        let accounts = WithdrawAccounts::try_from(accounts)?;
        let data = WithdrawInstructionData::try_from(data)?;
        Ok(Self { accounts, data })
    }

    /// 执行取款逻辑
    pub fn process(&self, program_id: &Pubkey) -> ProgramResult {
        // 1. 验证 vault 是正确的 PDA 并获取 bump
        let seeds = &[b"vault", self.accounts.owner.key().as_ref()];
        let bump = verify_pda(self.accounts.vault, seeds, program_id)?;

        // 2. 验证 vault 已初始化且所有者正确
        let vault_data = self.accounts.vault.try_borrow_data()?;
        let vault = Vault::from_bytes(&vault_data)?;
        vault.check_initialized()?;
        vault.check_owner(self.accounts.owner.key().as_ref())?;
        drop(vault_data); // 释放借用

        // 3. 检查余额是否足够
        let vault_balance = self.accounts.vault.lamports();
        let rent_exempt = pinocchio_system::rent::Rent::get()?
            .minimum_balance(Vault::LEN);

        if vault_balance < self.data.amount + rent_exempt {
            return Err(VaultError::InsufficientBalance.into());
        }

        // 4. 执行带签名的转账(从 vault 到 owner)
        // 使用 PDA 签名
        let signer_seeds = &[
            b"vault",
            self.accounts.owner.key().as_ref(),
            &[bump],
        ];

        // 手动构建 CPI 调用
        transfer(
            self.accounts.vault,
            self.accounts.owner,
            self.data.amount,
            &[signer_seeds],
        )?;

        #[cfg(not(feature = "perf"))]
        pinocchio::msg!(
            "Withdrawn {} lamports from vault",
            self.data.amount
        );

        Ok(())
    }
}
