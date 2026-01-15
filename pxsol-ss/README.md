# pxsol-ss

一个 Solana 链上数据存储程序。

## 项目简介

`pxsol-ss` 是一个 Solana 程序（智能合约），用于在链上存储用户数据。程序支持：
- 为用户创建独立的 PDA（Program Derived Address）数据账户
- 存储和更新用户数据
- 自动管理租金（数据变长时补足，变短时退还）

## 项目结构

```
pxsol-ss/
├── Cargo.toml          # 项目配置文件
├── src/
│   └── lib.rs          # Solana 程序主文件
├── README.md           # 项目说明文档
├── NOTES.md            # 学习笔记与参考资料
├── ACCOUNT_STRUCTURE.md # 账户结构说明
├── PROGRAM_ID_EXPLANATION.md # Program ID 说明
└── PROGRAM_PERMISSIONS.md # 程序权限说明
```

## 环境要求

- Rust 1.70.0 或更高版本
- Solana CLI（包含 cargo-build-sbf）
- Solana 开发工具链

### 安装 Solana 开发环境

```bash
curl --proto '=https' --tlsv1.2 -sSfL https://solana-install.solana.workers.dev | bash
```

确保 `cargo-build-sbf` 在 PATH 中，或使用完整路径。

## 构建项目

### 编译 Solana 程序

```bash
# 使用 cargo-build-sbf 编译（推荐）
cargo-build-sbf -- -Znext-lockfile-bump

# 或者使用完整路径
/home/dmin/.local/share/solana/install/active_release/bin/cargo-build-sbf -- -Znext-lockfile-bump
```

编译成功后，在 `target/deploy/` 目录下会生成 `pxsol_ss.so` 文件。

### 普通 Rust 构建（用于测试）

```bash
# 构建项目（用于单元测试）
cargo build

# 构建发布版本
cargo build --release
```

## 本地测试

### 1. 启动本地验证器

```bash
# 在终端 1 启动本地 Solana 验证器
solana-test-validator
```

### 2. 配置 CLI 连接到本地网络

```bash
# 在终端 2 配置 CLI
solana config set --url localhost

# 检查配置
solana config get
```

### 3. 部署程序

```bash
# 部署到本地验证器
solana program deploy target/deploy/pxsol_ss.so

# 部署成功后会显示程序 ID
```

### 4. 测试程序

参考 `NOTES.md` 中的 Python 示例代码，或使用 Solana CLI 和 JavaScript/TypeScript 客户端进行测试。

## 运行测试

```bash
# 运行所有测试
cargo test
```

## 项目功能

- **创建数据账户**：为用户创建独立的 PDA 数据账户
- **存储数据**：将用户数据存储到链上
- **更新数据**：支持更新已存储的数据
- **自动租金管理**：数据变长时自动补足租金，变短时自动退还

## 开发说明

本项目遵循以下开发原则：
- SOLID 原则
- DRY 原则（Don't Repeat Yourself）
- KISS 原则（Keep It Simple, Stupid）
- YAGNI 原则（You Aren't Gonna Need It）
- OWASP 最佳实践

## 版本信息

- 当前版本：0.1.0
- Rust Edition：2021
- Solana Program SDK：2.x

## 参考资料

- [NOTES.md](NOTES.md) - 详细的学习笔记和教程
- [ACCOUNT_STRUCTURE.md](ACCOUNT_STRUCTURE.md) - 账户结构说明
- [PROGRAM_ID_EXPLANATION.md](PROGRAM_ID_EXPLANATION.md) - Program ID 详解
- [PROGRAM_PERMISSIONS.md](PROGRAM_PERMISSIONS.md) - 程序权限说明

## 许可证

[待添加]

## 贡献

欢迎提交 Issue 和 Pull Request。
