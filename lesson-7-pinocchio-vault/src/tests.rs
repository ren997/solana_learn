use pinocchio::pubkey::Pubkey;

#[test]
fn test_vault_structure() {
    use crate::state::Vault;
    
    // 验证 Vault 结构大小
    assert_eq!(Vault::LEN, 34);
    assert_eq!(core::mem::size_of::<Vault>(), 34);
}

#[test]
fn test_discriminators() {
    use crate::instructions::{Initialize, Deposit, Withdraw};
    
    // 验证判别器唯一性
    assert_eq!(Initialize::DISCRIMINATOR, 0);
    assert_eq!(Deposit::DISCRIMINATOR, 1);
    assert_eq!(Withdraw::DISCRIMINATOR, 2);
}

// 注意: 完整的集成测试需要 mollusk-svm 和 solana-sdk
// 这里提供基本的单元测试框架
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    // TODO: 添加 Mollusk 集成测试
    // 需要安装并配置 mollusk-svm
    
    #[test]
    #[ignore] // 暂时忽略,需要完整的测试环境
    fn test_initialize_vault() {
        // 测试初始化金库
        // let program_id = Pubkey::new_unique();
        // let owner = Pubkey::new_unique();
        // ...
    }
    
    #[test]
    #[ignore]
    fn test_deposit() {
        // 测试存款功能
    }
    
    #[test]
    #[ignore]
    fn test_withdraw() {
        // 测试取款功能
    }
}
