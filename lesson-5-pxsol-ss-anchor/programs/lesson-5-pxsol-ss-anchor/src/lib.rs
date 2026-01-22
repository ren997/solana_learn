use anchor_lang::prelude::*;

// 程序 ID（部署后的链上地址）
declare_id!("5e44g7KZvJuhEPEuYX6S8tHWtb2FEyCg41HvDYwwV7z5");

// PDA 派生种子：固定字符串 "data"
const SEED: &[u8] = b"data";

// ============================================
// 指令定义：每个 pub fn 都是一条可调用的指令
// ============================================
#[program]
pub mod lesson_5_pxsol_ss_anchor {
    use super::*;

    /// 指令 1：初始化 PDA 账户
    /// - 创建用户的数据存储账户
    /// - 记录所有者和 bump 值
    pub fn init(ctx: Context<Init>) -> Result<()> {
        let account_user = &ctx.accounts.user;
        let account_user_pda = &mut ctx.accounts.user_pda;
        
        // 记录账户所有者
        account_user_pda.auth = account_user.key();
        // 记录 PDA 的 bump 值（Anchor 自动计算）
        account_user_pda.bump = ctx.bumps.user_pda;
        // 初始化数据为空
        account_user_pda.data = Vec::new();
        Ok(())
    }

    /// 指令 2：更新数据
    /// - 支持账户扩容/缩小
    /// - 缩小时自动退还多余租金
    /// 
    /// 参数位置匹配：
    /// - 位置 1: data (Vec<u8>) ← 按顺序对应 #[instruction] 的声明
    pub fn update(ctx: Context<Update>, data: Vec<u8>) -> Result<()> {
        let account_user = &ctx.accounts.user;
        let account_user_pda = &mut ctx.accounts.user_pda;

        // 更新数据内容
        account_user_pda.data = data;

        // 账户缩小时，退还多余的租金
        // （扩容时 Anchor 自动补缴租金，但缩小时需要手动退款）
        let account_user_pda_info = account_user_pda.to_account_info();
        let rent_exemption = Rent::get()?.minimum_balance(account_user_pda_info.data_len());
        let hold = **account_user_pda_info.lamports.borrow();
        
        if hold > rent_exemption {
            let refund = hold.saturating_sub(rent_exemption);
            **account_user_pda_info.lamports.borrow_mut() = rent_exemption;
            **account_user.lamports.borrow_mut() = account_user.lamports().checked_add(refund).unwrap();
        }
        Ok(())
    }

    /// 指令 3：关闭账户
    /// - 删除 PDA 账户
    /// - 退还所有租金给用户
    /// - 实际关闭由 #[account(close = user)] 约束自动处理
    pub fn close(_ctx: Context<Close>) -> Result<()> {
        Ok(())
    }
}

// ============================================
// 账户数据结构定义
// ============================================

/// PDA 账户的数据结构
/// #[account] 宏会自动添加 8 字节的 discriminator 用于类型识别
#[account]
pub struct Data {
    pub auth: Pubkey,  // 账户所有者（32 字节）
    pub bump: u8,      // PDA 的 bump 值（1 字节）
    pub data: Vec<u8>  // 用户存储的数据（4字节长度 + 实际数据）
}

impl Data {
    /// 计算账户所需空间
    /// 结构：discriminator(8) + auth(32) + bump(1) + vec_len(4) + data
    pub fn space_for(data_len: usize) -> usize {
        8 + 32 + 1 + 4 + data_len
    }
}

// ============================================
// 账户约束定义：init 指令
// ============================================

/// Init 指令的账户列表
/// - 必须用 #[derive(Accounts)] 标记
/// - 必须有生命周期参数 'info
#[derive(Accounts)]
pub struct Init<'info> {
    /// 调用者账户
    #[account(mut)]  // mut = 可写（需要扣除租金）
    pub user: Signer<'info>,  // Signer = 需要签名
    
    /// 要创建的 PDA 账户
    #[account(
        init,                                    // 标记为新建账户
        payer = user,                            // 租金由 user 支付
        seeds = [SEED, user.key().as_ref()],    // PDA 种子：["data", 用户公钥]
        bump,                                    // Anchor 自动计算 bump
        space = Data::space_for(0)              // 分配空间（初始数据为空）
    )]
    pub user_pda: Account<'info, Data>,
    
    /// 系统程序（用于创建账户）
    pub system_program: Program<'info, System>,
}

// ============================================
// 账户约束定义：update 指令
// ============================================

/// Update 指令的账户列表
/// 
/// 参数声明规则（按位置顺序匹配）：
/// #[instruction(new_data: Vec<u8>)]
///               ^^^^^^^^^^^^^^^^^ 
///               ↑ 声明位置 1
///               ↓ 对应指令函数的位置 1 参数 (data: Vec<u8>)
/// 
/// 匹配机制：
/// - 客户端调用：program.methods.update(Buffer.from("..."))
/// - 指令数据（instruction_data）：
///   - 前 8 字节：update 指令的 discriminator（用来标识调用哪个指令）
///   - 后续字节：data 的 Borsh 编码（Vec<u8> = len(u32 小端) + bytes）
/// - Anchor解析： 位置1的数据 → new_data
/// - 约束使用：  new_data.len() 计算空间
#[derive(Accounts)]
#[instruction(new_data: Vec<u8>)]  // 声明需要访问指令参数（用于计算 realloc）
pub struct Update<'info> {
    /// 调用者账户
    #[account(mut)]  // 可写（可能需要补缴或退还租金）
    pub user: Signer<'info>,
    
    /// 要更新的 PDA 账户
    #[account(
        mut,                                     // 可写（需要修改数据）
        seeds = [SEED, user.key().as_ref()],    // 验证 PDA 派生规则
        bump = user_pda.bump,                    // 使用存储的 bump 值
        realloc = Data::space_for(new_data.len()),  // 动态调整空间（使用声明的参数）
        realloc::payer = user,                   // 扩容时由 user 补缴租金
        realloc::zero = false,                   // 不清零新空间（节省计算单元）
        constraint = user_pda.auth == user.key() @ PxsolError::Unauthorized,  // 权限检查
    )]
    pub user_pda: Account<'info, Data>,
    
    /// 系统程序（用于重新分配空间和转账）
    pub system_program: Program<'info, System>,
}

// ============================================
// 账户约束定义：close 指令
// ============================================

/// Close 指令的账户列表
#[derive(Accounts)]
pub struct Close<'info> {
    /// 调用者账户（接收退还的租金）
    #[account(mut)]
    pub user: Signer<'info>,
    
    /// 要关闭的 PDA 账户
    #[account(
        mut,                                     // 可写
        seeds = [SEED, user.key().as_ref()],    // 验证 PDA
        bump = user_pda.bump,                    // 验证 bump
        close = user,                            // 关闭账户，租金退还给 user
        constraint = user_pda.auth == user.key() @ PxsolError::Unauthorized,  // 权限检查
    )]
    pub user_pda: Account<'info, Data>,
}

// ============================================
// 错误定义
// ============================================

#[error_code]
pub enum PxsolError {
    #[msg("Unauthorized: only the account owner can update data")]
    Unauthorized,
}
