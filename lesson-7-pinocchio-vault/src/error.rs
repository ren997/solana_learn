use {
    num_derive::FromPrimitive,
    pinocchio::program_error::ProgramError,
    thiserror::Error,
};

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum VaultError {
    /// 账户不是签名者
    #[error("Account is not a signer")]
    NotSigner,

    /// 账户所有者无效
    #[error("Invalid account owner")]
    InvalidOwner,

    /// 账户数据无效
    #[error("Invalid account data")]
    InvalidAccountData,

    /// 金额必须大于零
    #[error("Amount must be greater than zero")]
    InvalidAmount,

    /// 余额不足
    #[error("Insufficient balance")]
    InsufficientBalance,

    /// 金库已初始化
    #[error("Vault already initialized")]
    AlreadyInitialized,

    /// 金库未初始化
    #[error("Vault not initialized")]
    NotInitialized,
}

impl From<VaultError> for ProgramError {
    fn from(e: VaultError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
