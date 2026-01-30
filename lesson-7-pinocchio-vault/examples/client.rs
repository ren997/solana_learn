// 这是一个示例客户端,展示如何与 Vault 程序交互
// 注意: 这需要额外的依赖,仅作为参考

use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};

/// 程序 ID (部署后替换为实际的 Program ID)
const PROGRAM_ID: &str = "YourProgramIdHere";

/// 指令判别器
mod discriminator {
    pub const INITIALIZE: u8 = 0;
    pub const DEPOSIT: u8 = 1;
    pub const WITHDRAW: u8 = 2;
}

/// 派生 Vault PDA
fn derive_vault_pda(owner: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"vault", owner.as_ref()], program_id)
}

/// 创建初始化指令
fn create_initialize_instruction(
    owner: &Pubkey,
    program_id: &Pubkey,
) -> Instruction {
    let (vault_pda, _bump) = derive_vault_pda(owner, program_id);

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*owner, true),              // owner (signer, writable)
            AccountMeta::new(vault_pda, false),          // vault PDA (writable)
            AccountMeta::new_readonly(system_program::ID, false), // system program
        ],
        data: vec![discriminator::INITIALIZE],
    }
}

/// 创建存款指令
fn create_deposit_instruction(
    owner: &Pubkey,
    amount: u64,
    program_id: &Pubkey,
) -> Instruction {
    let (vault_pda, _bump) = derive_vault_pda(owner, program_id);

    let mut data = vec![discriminator::DEPOSIT];
    data.extend_from_slice(&amount.to_le_bytes());

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*owner, true),              // owner (signer, writable)
            AccountMeta::new(vault_pda, false),          // vault PDA (writable)
            AccountMeta::new_readonly(system_program::ID, false), // system program
        ],
        data,
    }
}

/// 创建取款指令
fn create_withdraw_instruction(
    owner: &Pubkey,
    amount: u64,
    program_id: &Pubkey,
) -> Instruction {
    let (vault_pda, _bump) = derive_vault_pda(owner, program_id);

    let mut data = vec![discriminator::WITHDRAW];
    data.extend_from_slice(&amount.to_le_bytes());

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*owner, true),              // owner (signer, writable)
            AccountMeta::new(vault_pda, false),          // vault PDA (writable)
            AccountMeta::new_readonly(system_program::ID, false), // system program
        ],
        data,
    }
}

/// 示例: 完整的工作流程
#[cfg(feature = "example")]
fn example_workflow() {
    use solana_client::rpc_client::RpcClient;
    
    // 连接到本地测试网
    let rpc_url = "http://localhost:8899";
    let client = RpcClient::new(rpc_url.to_string());
    
    // 加载用户密钥对
    let owner = Keypair::new();
    let program_id = PROGRAM_ID.parse::<Pubkey>().unwrap();
    
    // 1. 初始化金库
    println!("初始化金库...");
    let init_ix = create_initialize_instruction(&owner.pubkey(), &program_id);
    let recent_blockhash = client.get_latest_blockhash().unwrap();
    let init_tx = Transaction::new_signed_with_payer(
        &[init_ix],
        Some(&owner.pubkey()),
        &[&owner],
        recent_blockhash,
    );
    client.send_and_confirm_transaction(&init_tx).unwrap();
    println!("✓ 金库初始化成功");
    
    // 2. 存款
    println!("存款 1 SOL...");
    let deposit_amount = 1_000_000_000; // 1 SOL = 10^9 lamports
    let deposit_ix = create_deposit_instruction(&owner.pubkey(), deposit_amount, &program_id);
    let recent_blockhash = client.get_latest_blockhash().unwrap();
    let deposit_tx = Transaction::new_signed_with_payer(
        &[deposit_ix],
        Some(&owner.pubkey()),
        &[&owner],
        recent_blockhash,
    );
    client.send_and_confirm_transaction(&deposit_tx).unwrap();
    println!("✓ 存款成功");
    
    // 3. 取款
    println!("取款 0.5 SOL...");
    let withdraw_amount = 500_000_000; // 0.5 SOL
    let withdraw_ix = create_withdraw_instruction(&owner.pubkey(), withdraw_amount, &program_id);
    let recent_blockhash = client.get_latest_blockhash().unwrap();
    let withdraw_tx = Transaction::new_signed_with_payer(
        &[withdraw_ix],
        Some(&owner.pubkey()),
        &[&owner],
        recent_blockhash,
    );
    client.send_and_confirm_transaction(&withdraw_tx).unwrap();
    println!("✓ 取款成功");
    
    // 4. 查询金库余额
    let (vault_pda, _) = derive_vault_pda(&owner.pubkey(), &program_id);
    let vault_account = client.get_account(&vault_pda).unwrap();
    println!("金库余额: {} lamports", vault_account.lamports);
}

fn main() {
    println!("Pinocchio Vault 客户端示例");
    println!("============================");
    println!();
    println!("这是一个示例客户端代码,展示如何与 Vault 程序交互。");
    println!();
    println!("使用方法:");
    println!("1. 部署程序并获取 Program ID");
    println!("2. 替换 PROGRAM_ID 常量");
    println!("3. 添加必要的依赖到 Cargo.toml:");
    println!("   solana-sdk = \"2.0\"");
    println!("   solana-client = \"2.0\"");
    println!("4. 运行: cargo run --example client --features example");
    println!();
    println!("指令格式:");
    println!("- Initialize: [0]");
    println!("- Deposit:    [1, amount (8 bytes)]");
    println!("- Withdraw:   [2, amount (8 bytes)]");
}
