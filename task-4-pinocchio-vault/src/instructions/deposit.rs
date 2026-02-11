use core::mem::size_of;
use pinocchio::{AccountView, Address, ProgramResult, error::ProgramError};
use pinocchio_system::instructions::Transfer;

pub struct DepositAccounts<'a> {
    pub owner: &'a AccountView,
    pub vault: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for DepositAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        // 这里我们解构前两个账号，第三个账号用_忽略，
        // 如果账户数量不足则返回错误
        // let else 语法是 Rust 中的模式匹配语法，用于解构数组或切片，
        // 如果解构成功，则将解构后的值赋给左边的变量，如果解构失败，则执行 else 块中的代码。
        let [owner, vault, _rest @ ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // 账户验证，owner 必须是签名者
        if !owner.is_signer() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // 账户验证，vault 必须是系统程序所有者
        if !vault.owned_by(&pinocchio_system::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // 账户验证，vault 必须是空账户
        if vault.lamports().ne(&0) {
            return Err(ProgramError::InvalidAccountData);
        }
 
        // 账户验证，vault 必须是 PDA 账户
        let (vault_key, _) = Address::find_program_address(&[b"vault", owner.address().as_ref()], &crate::ID);
        if vault.address().ne(&vault_key) {
            return Err(ProgramError::InvalidAccountOwner);
        }
        Ok(DepositAccounts { owner, vault })
    }
}

pub struct DepositInstructionData { pub amount: u64 }

impl<'a> TryFrom<&'a [u8]> for DepositInstructionData {
    type Error = ProgramError;
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        // 验证数据长度
        if data.len() != size_of::<u64>() {
            return Err(ProgramError::InvalidInstructionData);
        }
        // 将数据转换为 u64
        let mut array = [0u8; size_of::<u64>()];
        array.copy_from_slice(&data[0..size_of::<u64>()]);
        let amount = u64::from_le_bytes(array);
        // 验证金额
        if amount.eq(&0) {
            return Err(ProgramError::InvalidInstructionData);
        }
        // 返回指令数据
        Ok(DepositInstructionData { amount })
    }
}


pub struct Deposit<'a> {
    pub accounts: DepositAccounts<'a>,
    pub instruction_data: DepositInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountView])> for Deposit<'a> {
    type Error = ProgramError;
    fn try_from((data, accounts): (&'a [u8], &'a [AccountView])) -> Result<Self, Self::Error> {
        let accounts = DepositAccounts::try_from(accounts)?;
        let instruction_data = DepositInstructionData::try_from(data)?;
        Ok(Self { accounts, instruction_data })
    }
}

impl<'a> Deposit<'a> {
    pub const DISCRIMINATOR: u8 = 0;
    pub fn process(&self) -> ProgramResult {
        Transfer {
            from: self.accounts.owner,
            to: self.accounts.vault,
            lamports: self.instruction_data.amount,
        }
        .invoke()?;
        Ok(())
    }
}