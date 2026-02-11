use pinocchio::{AccountView, Address, ProgramResult, address::declare_id, error::ProgramError, entrypoint};
mod instructions;

use instructions::Deposit;

use instructions::Withdraw;

declare_id!("22222222222222222222222222222222222222222222");

entrypoint!(process_instruction);

fn process_instruction(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    match instruction_data.split_first() {
        Some((&Deposit::DISCRIMINATOR, data)) => {
            Deposit::try_from((data, accounts))?.process()
        }
        Some((&Withdraw::DISCRIMINATOR, _)) => {
            Withdraw::try_from(accounts)?.process()
        }
        _ => Err(ProgramError::InvalidInstructionData),
    }
}