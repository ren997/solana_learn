# Pinocchio Vault 快速入门

## 快速开始

### 1. 检查环境

确保已安装以下工具:

```bash
# 检查 Rust
rustc --version

# 检查 Solana CLI
solana --version

# 检查 cargo-build-bpf (或 cargo-build-sbf)
cargo build-bpf --version
# 或
cargo build-sbf --version
```

### 2. 构建程序

```bash
cd solana_learn/lesson-7-pinocchio-vault

# 构建程序 (生产版本)
cargo build-bpf --release

# 或使用新版本命令
cargo build-sbf --release
```

构建成功后,程序文件位于: `target/deploy/vault_program.so`

### 3. 运行测试

```bash
# 运行基础单元测试
cargo test

# 运行 Solana 程序测试 (需要 Solana 测试环境)
cargo test-sbf
```

### 4. 部署到本地测试网

```bash
# 启动本地测试验证器
solana-test-validator

# 在另一个终端部署程序
solana program deploy target/deploy/vault_program.so

# 记录返回的 Program ID
```

## 项目结构说明

```
vault-program/
├── Cargo.toml              # 项目配置和依赖
├── README.md               # 详细文档
├── QUICKSTART.md          # 本文件
├── .gitignore             # Git 忽略文件
└── src/
    ├── lib.rs             # 程序入口点 (entrypoint)
    ├── error.rs           # 自定义错误类型
    ├── helpers.rs         # 账户验证辅助函数
    ├── tests.rs           # 测试模块
    ├── state/             # 账户状态定义
    │   ├── mod.rs
    │   └── vault.rs       # Vault 数据结构
    └── instructions/      # 指令实现
        ├── mod.rs
        ├── initialize.rs  # 初始化金库
        ├── deposit.rs     # 存款
        └── withdraw.rs    # 取款
```

## 核心文件说明

### lib.rs - 程序入口
- 定义 `entrypoint!` 宏
- 实现 `process_instruction` 函数
- 根据 discriminator 路由到不同指令

### error.rs - 错误处理
- 使用 `thiserror` 定义自定义错误
- 实现 `From<VaultError> for ProgramError`

### helpers.rs - 验证辅助
- 定义 `AccountCheck` trait
- 实现 `SignerAccount` 和 `SystemAccount`
- 提供 PDA 验证函数

### state/vault.rs - 状态管理
- 定义 Vault 数据结构 (34 字节)
- 实现零拷贝反序列化
- 提供初始化和验证方法

### instructions/ - 指令实现
每个指令文件包含:
1. Accounts 结构 - 定义所需账户
2. InstructionData 结构 - 定义指令数据
3. Instruction 结构 - 组合账户和数据
4. TryFrom 实现 - 验证逻辑
5. process 方法 - 业务逻辑

## 关键概念

### 1. Discriminator (判别器)
每个指令使用一个字节标识:
- Initialize: 0
- Deposit: 1
- Withdraw: 2

### 2. TryFrom Pattern
用于类型安全的验证和转换:
```rust
impl<'a> TryFrom<&'a [AccountInfo]> for DepositAccounts<'a> {
    // 验证账户
}
```

### 3. 零拷贝 (Zero-Copy)
直接从字节切片读取数据:
```rust
let vault = unsafe { &*(data.as_ptr() as *const Vault) };
```

### 4. PDA (Program Derived Address)
使用种子派生地址:
```rust
seeds = [b"vault", owner_pubkey]
```

## 调试技巧

### 启用调试日志

```bash
# 构建调试版本
cargo build-bpf --no-default-features

# 查看程序日志
solana logs
```

### 常见问题

1. **编译错误**: 确保 Rust 版本 >= 1.70
2. **依赖问题**: 运行 `cargo update`
3. **测试失败**: 检查 Solana 版本兼容性

## 下一步

1. 阅读 `README.md` 了解详细文档
2. 查看 `notes.md` 学习 Pinocchio 概念
3. 修改代码添加新功能
4. 编写更多测试用例

## 学习路径

1. **理解入口点**: 从 `lib.rs` 开始
2. **学习验证**: 查看 `helpers.rs` 和各指令的 `TryFrom`
3. **掌握状态管理**: 研究 `state/vault.rs`
4. **实现业务逻辑**: 分析各指令的 `process` 方法
5. **优化性能**: 理解 feature flags 和零拷贝

## 参考资源

- [Pinocchio 文档](https://github.com/febo/pinocchio)
- [Solana 开发者文档](https://docs.solana.com/developers)
- [Rust 官方教程](https://doc.rust-lang.org/book/)

祝学习愉快! 🚀
