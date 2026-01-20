#![allow(unexpected_cfgs)]

use solana_program::sysvar::Sysvar;

// 定义程序入口点
solana_program::entrypoint!(process_instruction);

/// 铸造指令处理函数
/// 
/// 功能：为 Ada 的账户铸造新的泰铢币（增发货币）
/// 
/// 参数：
/// - program_id: 当前程序的公钥
/// - accounts: 账户列表
///   [0] 铸造权限所有者的钱包账户（需要签名，可写）
///   [1] 铸造权限所有者的 PDA 数据账户（可写）
///   [2] System 程序
///   [3] Sysvar Rent 程序
/// - data: 指令数据，包含要铸造的代币数量（u64，大端序）
pub fn process_instruction_mint(
    program_id: &solana_program::pubkey::Pubkey,
    accounts: &[solana_program::account_info::AccountInfo],
    data: &[u8],
) -> solana_program::entrypoint::ProgramResult {
    // 解析账户列表
    let accounts_iter = &mut accounts.iter();
    let account_user = solana_program::account_info::next_account_info(accounts_iter)?;         // 用户钱包账户
    let account_user_pda = solana_program::account_info::next_account_info(accounts_iter)?;     // 用户的 PDA 数据账户
    let _ = solana_program::account_info::next_account_info(accounts_iter)?;                    // System 程序
    let _ = solana_program::account_info::next_account_info(accounts_iter)?;                    // Sysvar Rent 程序

    // 权限检查
    assert!(account_user.is_signer);  // 确保用户账户已签名
    // 只有 Ada 可以铸造泰铢币（硬编码的铸造权限控制）
    assert_eq!(*account_user.key, solana_program::pubkey!("6ASf5EcmmEHTgDJ4X4ZT5vT6iHVJBXPg5AN5YoTCpGWt"));
    // 计算并验证 PDA 地址是否正确
    let account_user_pda_calc =
        solana_program::pubkey::Pubkey::find_program_address(&[&account_user.key.to_bytes()], program_id);
    assert_eq!(account_user_pda.key, &account_user_pda_calc.0);

    // 如果 PDA 数据账户尚未初始化，则创建它
    if **account_user_pda.try_borrow_lamports().unwrap() == 0 {
        // 计算租金豁免所需的最小余额（8 字节数据）
        let rent_exemption = solana_program::rent::Rent::get()?.minimum_balance(8);
        let bump = account_user_pda_calc.1;  // 获取 PDA 的 bump seed
        // 调用 System 程序创建账户（使用 invoke_signed 因为需要 PDA 签名）
        solana_program::program::invoke_signed(
            &solana_program::system_instruction::create_account(
                account_user.key,        // 付款账户
                account_user_pda.key,    // 新账户地址（PDA）
                rent_exemption,          // 转账金额（租金豁免）
                8,                       // 数据大小（8 字节用于存储 u64 余额）
                program_id,              // 账户所有者（当前程序）
            ),
            accounts,
            &[&[&account_user.key.to_bytes(), &[bump]]],  // PDA 签名种子
        )?;
        // 初始化余额为 0（u64::MIN = 0）
        account_user_pda.data.borrow_mut().copy_from_slice(&u64::MIN.to_be_bytes());
    }

    // 执行铸造操作
    let mut buf = [0u8; 8];
    // 读取当前余额
    buf.copy_from_slice(&account_user_pda.data.borrow());
    let old = u64::from_be_bytes(buf);  // 从大端序字节数组解析为 u64
    // 读取要铸造的数量
    buf.copy_from_slice(&data);
    let inc = u64::from_be_bytes(buf);  // 从指令数据解析铸造数量
    // 计算新余额（使用 checked_add 防止溢出）
    let new = old.checked_add(inc).unwrap();
    // 将新余额写回 PDA 数据账户
    account_user_pda.data.borrow_mut().copy_from_slice(&new.to_be_bytes());
    Ok(())
}

/// 转账指令处理函数
/// 
/// 功能：在两个账户之间转移泰铢币
/// 
/// 参数：
/// - program_id: 当前程序的公钥
/// - accounts: 账户列表
///   [0] 发送者的钱包账户（需要签名，可写）
///   [1] 发送者的 PDA 数据账户（可写）
///   [2] 接收者的钱包账户
///   [3] 接收者的 PDA 数据账户（可写）
///   [4] System 程序
///   [5] Sysvar Rent 程序
/// - data: 指令数据，包含转账金额（u64，大端序）
pub fn process_instruction_transfer(
    program_id: &solana_program::pubkey::Pubkey,
    accounts: &[solana_program::account_info::AccountInfo],
    data: &[u8],
) -> solana_program::entrypoint::ProgramResult {
    // 解析账户列表
    let accounts_iter = &mut accounts.iter();
    let account_user = solana_program::account_info::next_account_info(accounts_iter)?;         // 发送者钱包账户
    let account_user_pda = solana_program::account_info::next_account_info(accounts_iter)?;     // 发送者的 PDA 数据账户
    let account_into = solana_program::account_info::next_account_info(accounts_iter)?;         // 接收者钱包账户
    let account_into_pda = solana_program::account_info::next_account_info(accounts_iter)?;     // 接收者的 PDA 数据账户
    let _ = solana_program::account_info::next_account_info(accounts_iter)?;                    // System 程序
    let _ = solana_program::account_info::next_account_info(accounts_iter)?;                    // Sysvar Rent 程序

    // 权限检查
    assert!(account_user.is_signer);  // 确保发送者已签名
    // 验证发送者的 PDA 地址是否正确（防止他人盗用发送者的余额账户）
    let account_user_pda_calc =
        solana_program::pubkey::Pubkey::find_program_address(&[&account_user.key.to_bytes()], program_id);
    assert_eq!(account_user_pda.key, &account_user_pda_calc.0);
    // 验证接收者的 PDA 地址是否正确
    let account_into_pda_calc =
        solana_program::pubkey::Pubkey::find_program_address(&[&account_into.key.to_bytes()], program_id);
    assert_eq!(account_into_pda.key, &account_into_pda_calc.0);

    // 如果接收者的 PDA 数据账户尚未初始化，则自动创建
    if **account_into_pda.try_borrow_lamports().unwrap() == 0 {
        // 计算租金豁免所需的最小余额
        let rent_exemption = solana_program::rent::Rent::get()?.minimum_balance(8);
        let bump = account_into_pda_calc.1;  // 获取接收者 PDA 的 bump seed
        // 调用 System 程序创建接收者的数据账户（由发送者支付租金）
        solana_program::program::invoke_signed(
            &solana_program::system_instruction::create_account(
                account_user.key,        // 付款账户（发送者）
                account_into_pda.key,    // 新账户地址（接收者的 PDA）
                rent_exemption,          // 转账金额（租金豁免）
                8,                       // 数据大小（8 字节）
                program_id,              // 账户所有者（当前程序）
            ),
            accounts,
            &[&[&account_into.key.to_bytes(), &[bump]]],  // 接收者 PDA 签名种子
        )?;
        // 初始化接收者余额为 0
        account_into_pda.data.borrow_mut().copy_from_slice(&u64::MIN.to_be_bytes());
    }

    // 执行转账操作
    let mut buf = [0u8; 8];
    // 读取发送者当前余额
    buf.copy_from_slice(&account_user_pda.data.borrow());
    let old_user = u64::from_be_bytes(buf);  // 发送者原余额
    // 读取接收者当前余额
    buf.copy_from_slice(&account_into_pda.data.borrow());
    let old_into = u64::from_be_bytes(buf);  // 接收者原余额
    // 读取转账金额
    buf.copy_from_slice(&data);
    let inc = u64::from_be_bytes(buf);  // 从指令数据解析转账金额
    // 计算新余额（checked_sub 和 checked_add 防止溢出和下溢）
    let new_user = old_user.checked_sub(inc).unwrap();  // 发送者新余额（减少）
    let new_into = old_into.checked_add(inc).unwrap();  // 接收者新余额（增加）
    // 将新余额写回各自的 PDA 数据账户
    account_user_pda.data.borrow_mut().copy_from_slice(&new_user.to_be_bytes());  // 更新发送者余额
    account_into_pda.data.borrow_mut().copy_from_slice(&new_into.to_be_bytes());  // 更新接收者余额
    Ok(())
}

/// 程序主入口函数（指令路由器）
/// 
/// 根据指令数据的第一个字节来决定执行哪个操作：
/// - 0x00: 铸造操作
/// - 0x01: 转账操作
/// 
/// 参数：
/// - program_id: 当前程序的公钥
/// - accounts: 账户列表（具体内容由子指令决定）
/// - data: 指令数据（第一个字节为指令类型，后续字节为指令参数）
pub fn process_instruction(
    program_id: &solana_program::pubkey::Pubkey,
    accounts: &[solana_program::account_info::AccountInfo],
    data: &[u8],
) -> solana_program::entrypoint::ProgramResult {
    // 确保指令数据至少有 1 个字节（用于识别指令类型）
    assert!(data.len() >= 1);
    // 根据第一个字节路由到不同的指令处理函数
    match data[0] {
        0x00 => process_instruction_mint(program_id, accounts, &data[1..]),      // 铸造指令
        0x01 => process_instruction_transfer(program_id, accounts, &data[1..]),  // 转账指令
        _ => unreachable!(),  // 其他值视为无效指令
    }
}