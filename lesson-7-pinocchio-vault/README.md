# Pinocchio Vault Program

这是一个使用 Pinocchio 框架构建的 Solana 原生程序示例,实现了一个简单的金库(Vault)功能。

## 项目结构

```
vault-program/
├── Cargo.toml
└── src/
    ├── lib.rs              # 程序入口点
    ├── error.rs            # 错误定义
    ├── helpers.rs          # 账户验证辅助工具
    ├── tests.rs            # 测试模块
    ├── state/              # 状态定义
    │   ├── mod.rs
    │   └── vault.rs        # Vault 账户结构
    └── instructions/       # 指令实现
        ├── mod.rs
        ├── initialize.rs   # 初始化金库
        ├── deposit.rs      # 存款
        └── withdraw.rs     # 取款
```

## 功能特性

### 1. 初始化金库 (Initialize)
- 创建一个 PDA 金库账户
- 设置金库所有者
- 存储 bump seed

### 2. 存款 (Deposit)
- 将 SOL 从用户账户转入金库
- 验证金库所有者
- 检查金额有效性

### 3. 取款 (Withdraw)
- 从金库取出 SOL 到用户账户
- 使用 PDA 签名授权转账
- 保持租金豁免余额

## 核心概念

### Pinocchio 特性
- **零拷贝**: 直接从字节切片读取数据,无需反序列化
- **极简设计**: 更少的计算单元消耗
- **手动验证**: 完全控制账户和数据验证逻辑

### TryFrom Pattern
使用 Rust 的 `TryFrom` trait 实现类型安全的验证:
```rust
impl<'a> TryFrom<&'a [AccountInfo]> for DepositAccounts<'a> {
    type Error = ProgramError;
    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // 验证逻辑
    }
}
```

### 账户验证 Traits
定义可重用的验证逻辑:
```rust
pub trait AccountCheck {
    fn check(account: &AccountInfo) -> Result<(), ProgramError>;
}
```

## 构建和测试

### 前置要求
- Rust 工具链
- Solana CLI 工具
- cargo-build-bpf (或 cargo-build-sbf)

### 构建程序

```bash
# 构建生产版本(启用性能优化)
cargo build-bpf --release

# 或使用新版本命令
cargo build-sbf --release

# 构建调试版本(包含日志)
cargo build-bpf --no-default-features
```

### 运行测试

```bash
# 运行单元测试
cargo test

# 运行 Solana 程序测试
cargo test-sbf
```

## 性能优化

### Feature Flags
项目使用 feature flags 控制日志输出:

```toml
[features]
default = ["perf"]
perf = []
```

- `perf` 启用时: 禁用日志,最大化性能
- `perf` 禁用时: 启用调试日志

### 代码中使用
```rust
#[cfg(not(feature = "perf"))]
pinocchio::msg!("Debug message");
```

## 指令格式

### Initialize (Discriminator: 0)
```
Accounts:
  0. [signer, writable] owner - 金库所有者
  1. [writable] vault - 金库 PDA
  2. [] system_program - 系统程序

Data: [0]
```

### Deposit (Discriminator: 1)
```
Accounts:
  0. [signer, writable] owner - 存款人
  1. [writable] vault - 金库 PDA
  2. [] system_program - 系统程序

Data: [1, amount (8 bytes, little-endian u64)]
```

### Withdraw (Discriminator: 2)
```
Accounts:
  0. [signer, writable] owner - 取款人
  1. [writable] vault - 金库 PDA
  2. [] system_program - 系统程序

Data: [2, amount (8 bytes, little-endian u64)]
```

## PDA 派生

金库 PDA 使用以下种子派生:
```rust
seeds = [b"vault", owner_pubkey]
```

## 安全考虑

1. **签名验证**: 所有需要授权的操作都验证签名者
2. **所有者检查**: 验证操作者是金库所有者
3. **PDA 验证**: 确保使用正确的 PDA 地址
4. **余额检查**: 取款时保持租金豁免余额
5. **金额验证**: 拒绝零金额操作

## 学习资源

- [Pinocchio GitHub](https://github.com/febo/pinocchio)
- [Solana 官方文档](https://docs.solana.com/)
- [Anchor 框架](https://www.anchor-lang.com/)

## 许可证

MIT License

## 作者

基于 @Blueshift 的 Pinocchio 教程实现
