use pinocchio::program_error::ProgramError;
use crate::error::VaultError;

/// 金库账户数据结构
/// 
/// 布局:
/// - is_initialized: 1 字节 (bool)
/// - owner: 32 字节 (Pubkey)
/// - bump: 1 字节 (u8)
/// 总计: 34 字节
#[repr(C)]
pub struct Vault {
    /// 是否已初始化
    pub is_initialized: bool,
    /// 金库所有者
    pub owner: [u8; 32],
    /// PDA bump seed
    pub bump: u8,
}

impl Vault {
    /// 金库账户数据大小
    pub const LEN: usize = 1 + 32 + 1; // 34 字节

    /// 从字节切片反序列化金库数据
    pub fn from_bytes(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.len() != Self::LEN {
            return Err(VaultError::InvalidAccountData.into());
        }
        
        // 使用零拷贝转换
        let vault = unsafe { &*(data.as_ptr() as *const Vault) };
        Ok(vault)
    }

    /// 从可变字节切片反序列化金库数据
    pub fn from_bytes_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if data.len() != Self::LEN {
            return Err(VaultError::InvalidAccountData.into());
        }
        
        // 使用零拷贝转换
        let vault = unsafe { &mut *(data.as_mut_ptr() as *mut Vault) };
        Ok(vault)
    }

    /// 初始化金库
    pub fn initialize(&mut self, owner: &[u8; 32], bump: u8) -> Result<(), ProgramError> {
        if self.is_initialized {
            return Err(VaultError::AlreadyInitialized.into());
        }

        self.is_initialized = true;
        self.owner.copy_from_slice(owner);
        self.bump = bump;

        Ok(())
    }

    /// 验证金库已初始化
    pub fn check_initialized(&self) -> Result<(), ProgramError> {
        if !self.is_initialized {
            return Err(VaultError::NotInitialized.into());
        }
        Ok(())
    }

    /// 验证所有者
    pub fn check_owner(&self, owner: &[u8; 32]) -> Result<(), ProgramError> {
        if &self.owner != owner {
            return Err(VaultError::InvalidOwner.into());
        }
        Ok(())
    }
}
