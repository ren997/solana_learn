#![allow(unexpected_cfgs)]

// 声明程序入口点
solana_program::entrypoint!(process_instruction);

pub fn process_instruction(
    _: &solana_program::pubkey::Pubkey,
    accounts: &[solana_program::account_info::AccountInfo],
    _: &[u8],
) -> solana_program::entrypoint::ProgramResult {
    // ========== 1. 提取账户信息 ==========
    let accounts_iter = &mut accounts.iter();
    let account_user = solana_program::account_info::next_account_info(accounts_iter)?;          // 用户账户
    let account_user_spla = solana_program::account_info::next_account_info(accounts_iter)?;     // 用户的关联代币账户
    let account_mana = solana_program::account_info::next_account_info(accounts_iter)?;          // Mana 程序账户
    let account_mana_auth = solana_program::account_info::next_account_info(accounts_iter)?;     // Mana 的 PDA 权限账户
    let account_mana_spla = solana_program::account_info::next_account_info(accounts_iter)?;     // Mana 的关联代币账户
    let account_mint = solana_program::account_info::next_account_info(accounts_iter)?;          // 代币铸造账户
    let _ = solana_program::account_info::next_account_info(accounts_iter)?;                     // 跳过账户
    let account_spl = solana_program::account_info::next_account_info(accounts_iter)?;           // SPL Token 2022 程序
    let _ = solana_program::account_info::next_account_info(accounts_iter)?;                     // 跳过账户

    // ========== 2. 账户验证 ==========
    // 验证用户签名
    assert!(account_user.is_signer);
    
    // 验证用户的关联代币账户地址正确
    let account_user_spla_calc = spl_associated_token_account::get_associated_token_address_with_program_id(
        &account_user.key,
        &account_mint.key,
        &spl_token_2022::id(),
    );
    assert_eq!(account_user_spla.key, &account_user_spla_calc);
    
    // 验证 Mana 的 PDA 权限账户地址正确
    let account_mana_auth_calc = solana_program::pubkey::Pubkey::find_program_address(&[&[]], account_mana.key);
    assert_eq!(account_mana_auth.key, &account_mana_auth_calc.0);
    
    // 验证 Mana 的关联代币账户地址正确
    let account_mana_spla_calc = spl_associated_token_account::get_associated_token_address_with_program_id(
        &account_mana_auth.key,
        &account_mint.key,
        &spl_token_2022::id(),
    );
    assert_eq!(account_mana_spla.key, &account_mana_spla_calc);
    
    // 验证 SPL 程序地址正确
    assert_eq!(account_spl.key, &spl_token_2022::id());

    // ========== 3. 创建用户的关联代币账户（如果不存在） ==========
    solana_program::program::invoke(
        &spl_associated_token_account::instruction::create_associated_token_account_idempotent(
            &account_user.key,
            &account_user.key,
            &account_mint.key,
            &account_spl.key,
        ),
        accounts,
    )?;
    
    // ========== 4. 从 Mana 向用户转账代币（使用 PDA 签名） ==========
    solana_program::program::invoke_signed(
        &spl_token_2022::instruction::transfer_checked(
            &account_spl.key,               // SPL 程序
            &account_mana_spla.key,         // 源账户（Mana 的代币账户）
            &account_mint.key,              // 代币铸造账户
            &account_user_spla.key,         // 目标账户（用户的代币账户）
            &account_mana_auth.key,         // 转账权限（Mana 的 PDA）
            &[],                            // 多签账户（无）
            5000000000,                     // 转账数量（5 * 10^9，按 9 位小数）
            9,                              // 小数位数
        )?,
        accounts,
        &[&[&[], &[account_mana_auth_calc.1]]],  // PDA 签名种子
    )?;

    Ok(())
}
