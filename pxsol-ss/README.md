# pxsol-ss

一个 Solana 相关的 Rust 库项目。

## 项目简介

`pxsol-ss` 是一个 Rust 库项目，用于 Solana 区块链开发。

## 项目结构

```
pxsol-ss/
├── Cargo.toml      # 项目配置文件
├── src/
│   └── lib.rs      # 库主文件
├── README.md       # 项目说明文档
└── NOTES.md        # 学习笔记与参考资料
```

## 环境要求

- Rust 1.70.0 或更高版本
- Cargo（Rust 包管理器）

## 构建项目

```bash
# 构建项目
cargo build

# 构建发布版本
cargo build --release
```

## 运行测试

```bash
# 运行所有测试
cargo test
```

## 使用方式

作为库使用：

```toml
[dependencies]
pxsol-ss = { path = "../pxsol-ss" }
```

## 开发说明

本项目遵循以下开发原则：
- SOLID 原则
- DRY 原则（Don't Repeat Yourself）
- KISS 原则（Keep It Simple, Stupid）
- YAGNI 原则（You Aren't Gonna Need It）
- OWASP 最佳实践

## 版本信息

- 当前版本：0.1.0
- Rust Edition：2024

## 许可证

[待添加]

## 贡献

欢迎提交 Issue 和 Pull Request。
