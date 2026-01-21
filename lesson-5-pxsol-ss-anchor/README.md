# Lesson 5: Simple Data Storage with Anchor

这是使用 Anchor 框架重写的简单链上数据存储程序，演示了如何使用 Anchor 实现 PDA 账户的创建、数据存储和更新。

## 功能

- **init**: 初始化一个 PDA 账户，记录所有者并准备存储空间
- **update**: 更新 PDA 账户中的数据，支持动态调整空间大小（realloc）

## 项目结构

```
.
├── programs/
│   └── lesson-5-pxsol-ss-anchor/
│       └── src/
│           └── lib.rs          # Anchor 智能合约代码
├── tests/
│   ├── lesson-5-pxsol-ss-anchor.ts   # TypeScript 测试（Anchor 框架）
│   └── pxsol-ss-anchor.py      # Python 测试（原生 pxsol 客户端）
├── Anchor.toml                 # Anchor 配置文件
└── README.md                   # 本文件
```

## 快速开始

### 1. 构建程序

```bash
anchor build
```

### 2. 启动本地验证器

```bash
solana-test-validator -r
```

### 3. 运行测试

#### 方式一：使用 Anchor 测试框架（TypeScript）

```bash
# 自动构建、部署并运行测试
anchor test --skip-local-validator

# 或者不加 --skip-local-validator 让 Anchor 自动启动本地链
anchor test
```

#### 方式二：使用 Python 客户端

```bash
# 先部署程序
anchor build
anchor deploy

# 运行 Python 测试
python tests/pxsol-ss-anchor.py init
python tests/pxsol-ss-anchor.py update "The quick brown fox jumps over the lazy dog"
python tests/pxsol-ss-anchor.py load
# 输出：The quick brown fox jumps over the lazy dog

python tests/pxsol-ss-anchor.py update "片云天共远, 永夜月同孤."
python tests/pxsol-ss-anchor.py load
# 输出：片云天共远, 永夜月同孤.
```

## 核心概念

### Data 结构体

```rust
#[account]
pub struct Data {
    pub auth: Pubkey,  // 所有者的公钥
    pub bump: u8,      // PDA 的 bump 值
    pub data: Vec<u8>  // 存储的数据内容
}
```

### 账户空间计算

```
总空间 = 8 (discriminator) + 32 (auth) + 1 (bump) + 4 (Vec长度) + data_len
```

### Init 指令

- 创建 PDA 账户
- 记录所有者（auth）
- 存储 bump 值
- 初始化为空数据

### Update 指令

- 校验所有者权限
- 动态重新分配账户空间（realloc）
- 更新数据内容
- 自动处理租金退款（账户缩小时）

## 关键约束说明

### Init 账户约束

- `init`: 创建新账户
- `payer = user`: 由 user 支付租金
- `seeds = [SEED, user.key().as_ref()]`: PDA 推导种子
- `bump`: 自动求解 bump 值
- `space = Data::space_for(0)`: 分配初始空间

### Update 账户约束

- `mut`: 账户可写
- `seeds / bump`: 校验 PDA 地址
- `realloc`: 动态调整空间
- `constraint = user_pda.auth == user.key()`: 权限检查

## 测试流程

TypeScript 测试会执行以下步骤：

1. 初始化一个空的 PDA 账户
2. 更新数据为英文句子（账户扩容）
3. 更新数据为中文句子（账户缩小，退还多余租金）
4. 验证每次更新后数据正确

## 注意事项

- PDA 种子是 `b"data" + user_pubkey`
- 账户缩小时需要手动退还多余的 lamports
- Update 指令会检查调用者是否是账户所有者
- 使用 `#[instruction(new_data: Vec<u8>)]` 在约束中访问指令参数

## 相关文档

详细说明请查看 `NOTES.md` 文件。
