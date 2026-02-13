use pinocchio::{AccountView, Address, ProgramResult, address::declare_id, error::ProgramError, entrypoint};
mod instructions;
pub mod errors;
pub use errors::*;

declare_id!("22222222222222222222222222222222222222222222");

entrypoint!(process_instruction);

fn process_instruction(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    match instruction_data.split_first() {
        Some((&Make::DISCRIMINATOR, data)) => {
            Make::try_from((data, accounts))?.process()
        }
    }
}