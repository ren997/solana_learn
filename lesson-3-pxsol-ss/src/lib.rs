#![allow(unexpected_cfgs)]

use solana_program::sysvar::Sysvar;

// 注册程序入口点
// 这个宏将 process_instruction 函数注册为 Solana 程序的入口点
// Solana 运行时在处理交易时会调用这个函数
solana_program::entrypoint!(process_instruction);

/// Solana 程序的入口函数
/// 
/// 这是所有 Solana 程序必须实现的统一入口点，类似于 HTTP 服务器的 handle_request 函数
/// 每次用户调用程序时，Solana 运行时都会调用这个函数来处理指令
/// 
/// # 参数说明
/// 
/// - `program_id`: 当前程序的公钥地址（Pubkey），这是程序的唯一标识符
///                  - 由 Solana 运行时自动传入，表示"这是哪个程序在被调用"
///                  - 用于验证 PDA 派生和程序身份
///                  - 在创建账户时，作为账户的 owner（所有者）
/// - `accounts`: 账户列表数组，包含本次调用所需的所有账户
///                调用方必须按照固定顺序传入账户，顺序错误会导致程序执行失败
/// - `data`: 指令数据，即用户要存储的字节数组
/// 
/// # 返回值
/// 
/// - `ProgramResult`: 程序执行结果，Ok(()) 表示成功，Err 表示失败
pub fn process_instruction(
    program_id: &solana_program::pubkey::Pubkey,
    accounts: &[solana_program::account_info::AccountInfo],
    data: &[u8],
) -> solana_program::entrypoint::ProgramResult {
    // ============================================
    // 账户解析部分
    // ============================================
    // 创建账户迭代器，用于按顺序提取账户
    // 注意：账户顺序必须严格按照以下顺序，否则会导致程序逻辑错误
    let accounts_iter = &mut accounts.iter();
    
    // 账户 0: 用户钱包账户（调用者的钱包）
    // - 角色：支付租金和交易费用
    // - 签名：必须签名（调用者需要签名授权）
    // - 可写：是（需要从该账户扣除 SOL）
    let account_user = solana_program::account_info::next_account_info(accounts_iter)?;
    
    // 验证用户钱包账户必须签名
    // 这是安全检查，确保只有账户所有者才能调用此程序
    // 虽然 Solana 运行时层面也会验证签名，但程序层面的验证是重要的安全实践
    if !account_user.is_signer {
        return Err(solana_program::program_error::ProgramError::MissingRequiredSignature);
    }
    
    // 账户 1: 用户数据账户（PDA - Program Derived Address）
    // - 角色：存储用户数据的账户，由程序派生和管理
    // - 签名：否（PDA 没有私钥，由程序通过签名种子控制）
    // - 可写：是（需要写入或更新数据）
    let account_data = solana_program::account_info::next_account_info(accounts_iter)?;
    
    // 账户 2: System Program（系统程序）
    // - 角色：用于创建账户的系统程序
    // - 签名：否
    // - 可写：否
    // 注意：虽然不使用，但必须按顺序提取，否则后续账户会错位
    let _ = solana_program::account_info::next_account_info(accounts_iter)?;
    
    // 账户 3: Sysvar Rent（租金系统变量）
    // - 角色：用于查询当前网络的租金信息
    // - 签名：否
    // - 可写：否
    // 注意：虽然不使用，但必须按顺序提取，否则后续账户会错位
    let _ = solana_program::account_info::next_account_info(accounts_iter)?;

    // ============================================
    // PDA 派生和验证部分
    // ============================================
    // 计算当前数据长度所需的租金豁免金额
    // 租金 = 数据长度 × 每字节租金费率
    let rent_exemption = solana_program::rent::Rent::get()?.minimum_balance(data.len());
    
    // 派生 PDA（Program Derived Address）
    // PDA 是基于用户钱包地址和程序 ID 确定性派生的地址
    // 每个用户钱包会生成唯一的 PDA，用于存储该用户的数据
    // 返回值：(Pubkey, u8) - (PDA 地址, bump_seed)
    let calculated_pda =
        solana_program::pubkey::Pubkey::find_program_address(&[&account_user.key.to_bytes()], program_id);
    
    // 验证传入的数据账户地址是否与计算出的 PDA 匹配
    // 这是安全检查，防止调用方传入错误的账户地址
    assert_eq!(account_data.key, &calculated_pda.0);
    
    // 保存 bump_seed，用于后续的 PDA 签名
    // bump_seed 是确保 PDA 地址有效的种子值
    let bump_seed = calculated_pda.1;

    // ============================================
    // 用户数据账户创建逻辑（首次调用）
    // ============================================
    // 重要理解：PDA 账户的所有权关系
    // - PDA 账户是为用户创建的：每个用户有自己独立的数据账户（PDA）
    // - 但 PDA 账户的所有者（owner）是程序：账户属于程序（program_id），不属于用户
    // - PDA 只是用来存储用户的数据：用户通过程序来读写自己的数据
    //
    // 所有权关系：
    //   - 程序账户（program_id）：已部署，固定，所有者是 BPF Loader
    //   - 用户数据账户（PDA）：为每个用户创建，所有者是程序（program_id）
    //   - 用户钱包账户：属于用户自己，用户拥有私钥
    //
    // 为什么 PDA 属于程序而不是用户？
    // - 程序需要控制 PDA 的访问权限和数据格式
    // - 只有程序可以修改 PDA 的数据（通过程序逻辑）
    // - 用户不能直接操作 PDA，必须通过程序调用
    //
    // 检查用户的数据账户是否存在：余额为 0 表示账户尚未创建
    if **account_data.try_borrow_lamports().unwrap() == 0 {
        // ============================================
        // Solana 跨程序调用（CPI - Cross-Program Invocation）
        // ============================================
        // invoke_signed 是 Solana 程序库提供的 API，用于跨程序调用
        // - 它允许当前程序调用其他程序（这里是系统程序）
        // - "signed" 表示需要程序签名，用于让程序代表 PDA 签名（因为 PDA 没有私钥）
        //
        // 调用流程：
        //   当前程序 → invoke_signed → 系统程序（System Program）→ create_account 指令
        //
        // 重要概念区分：支付者 vs 账户所有者
        // - 支付者（account_user.key）：用户从自己的钱包支付租金来创建账户
        //   就像你花钱租房子，钱是你出的
        // - 账户所有者（program_id）：创建的PDA账户属于程序，不属于用户
        //   就像你租的房子，虽然你付了租金，但房子的所有权属于房东（程序）
        // - 用户只能通过程序来使用这个账户，不能直接控制
        solana_program::program::invoke_signed(
            // 构建系统程序的 create_account 指令
            // system_instruction::create_account 是 Solana 系统程序提供的标准指令
            &solana_program::system_instruction::create_account(
                account_user.key,      // 支付者：用户钱包（用户从自己的钱包支付租金来创建账户）
                account_data.key,      // 新账户：用户的数据账户（PDA 地址，用于存储用户数据）
                rent_exemption,        // 初始余额：租金豁免金额（从用户钱包扣除，存入PDA账户）
                data.len() as u64,     // 账户数据空间大小（根据数据长度分配）
                program_id,            // 账户所有者：当前程序的公钥地址（program_id）
                                       // - program_id 是当前程序的唯一标识符，由 Solana 运行时传入
                                       // - 创建的 PDA 账户将属于这个程序（owner = program_id）
                                       // - 重要：PDA账户属于程序，不属于用户
                                       // - 虽然用户支付了租金，但账户的控制权属于程序
            ),
            // 参数2：账户列表，传递给被调用的程序（系统程序）
            accounts,
            // 参数3：签名种子数组，用于程序签名
            // - 这是 invoke_signed 的关键：程序使用种子为 PDA 生成签名
            // - 种子：[用户公钥字节, bump_seed]
            // - 系统会验证这些种子能否派生出 PDA 地址，从而验证程序有权代表 PDA 操作
            &[&[&account_user.key.to_bytes(), &[bump_seed]]],
        )?;
        
        // 将用户数据写入新创建的数据账户
        account_data.data.borrow_mut().copy_from_slice(data);
        return Ok(());
    }

    // ============================================
    // 账户更新逻辑（后续调用）
    // ============================================
    
    // 租金补足：如果新数据比旧数据长，需要补足额外的租金
    if rent_exemption > account_data.lamports() {
        // 使用 invoke 调用系统程序转账（普通转账，不需要程序签名）
        solana_program::program::invoke(
            &solana_program::system_instruction::transfer(
                account_user.key,                              // 从用户钱包
                account_data.key,                              // 到数据账户
                rent_exemption - account_data.lamports(),     // 转账金额：租金差额
            ),
            accounts,
        )?;
    }
    
    // 租金退款：如果新数据比旧数据短，退还多余的租金
    if rent_exemption < account_data.lamports() {
        // 直接修改账户余额（不需要 transfer 指令）
        // 因为程序是 PDA 的 owner，可以直接操作账户余额
        // 将多余的租金退还到用户钱包
        **account_user.lamports.borrow_mut() = account_user.lamports() + account_data.lamports() - rent_exemption;
        // 将数据账户余额设置为所需的租金金额
        **account_data.lamports.borrow_mut() = rent_exemption;
    }
    
    // 重新分配账户数据空间
    // resize() 只接受一个参数：新数据长度
    // 注意：resize() 会自动保留旧数据（如果新长度大于旧长度）
    account_data.resize(data.len())?;
    
    // 将新数据写入账户
    account_data.data.borrow_mut().copy_from_slice(data);

    // 返回成功
    Ok(())
}