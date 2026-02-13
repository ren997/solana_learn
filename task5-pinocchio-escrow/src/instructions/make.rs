use pinocchio::{Address, AccountView, ProgramResult};
use pinocchio::cpi::Seed;
use pinocchio::error::ProgramError;
use pinocchio_token::instructions::Transfer;
use crate::instructions::helpers::{AccountCheck, SignerAccount, MintInterface, AssociatedTokenAccount, AssociatedTokenAccountCheck, ProgramAccount, Escrow, ProgramAccountInit, AssociatedTokenAccountInit};

pub struct MakeAccounts<'info> {
    // 创建者账户
    pub maker:  &'info AccountView,

    // 托管账户
    pub escrow: &'info AccountView,

    // 代币 A 的 Mint 账户
    pub mint_a: &'info AccountView,

    // 代币 B 的 Mint 账户
    pub mint_b: &'info AccountView,

    // 创建者的代币 A ATA
    pub maker_ata_a: &'info AccountView,

    // 金库账户
    pub vault: &'info AccountView,

    // 系统程序 
    pub system_program: &'info AccountView,

    // 代币程序
    pub token_program: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for MakeAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        // 解构账户数组
        let [maker, escrow, mint_a, mint_b, maker_ata_a, vault, system_program, token_program, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // 验证 maker 是签名者
        SignerAccount::check(maker)?;

        // 验证 mint_a 是有效的 Mint 账户
        MintInterface::check(mint_a)?;

        // 验证 mint_b 是有效的 Mint 账户
        MintInterface::check(mint_b)?;

        // 验证 maker_ata_a 是正确的 ATA
        AssociatedTokenAccount::check(maker_ata_a, maker, mint_a, token_program)?;

        // 验证 vault 是正确的 ATA
        AssociatedTokenAccount::check(vault, escrow, mint_a, token_program)?;

        // 验证 system_program 是有效的系统程序
        SystemProgram::check(system_program)?;

        // 验证 token_program 是有效的代币程序
        TokenProgram::check(token_program)?;

        Ok(MakeAccounts {
            maker,
            escrow,
            mint_a,
            mint_b,
            maker_ata_a,
            vault,
            system_program,
            token_program,
        })
    }
}
