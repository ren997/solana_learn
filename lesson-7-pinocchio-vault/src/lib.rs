use pinocchio::{
    account_info::AccountInfo,
    entrypoint,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

mod error;
mod helpers;
mod instructions;
mod state;

use instructions::{Deposit, Initialize, Withdraw};

entrypoint!(process_instruction);

/// 程序入口点
/// 
/// 这是所有指令的统一入口,根据第一个字节(discriminator)
/// 来决定调用哪个具体的指令处理器
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // 提取判别器(第一个字节)
    let (discriminator, data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    // 根据判别器路由到对应的指令处理器
    match discriminator {
        // 0: 初始化金库
        &Initialize::DISCRIMINATOR => {
            Initialize::try_from(accounts)?.process(program_id)
        }
        
        // 1: 存款
        &Deposit::DISCRIMINATOR => {
            Deposit::try_from(data, accounts)?.process(program_id)
        }
        
        // 2: 取款
        &Withdraw::DISCRIMINATOR => {
            Withdraw::try_from(data, accounts)?.process(program_id)
        }
        
        // 未知指令
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

#[cfg(test)]
mod tests;
