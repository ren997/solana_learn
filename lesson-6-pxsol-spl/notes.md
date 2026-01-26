---
title: 第六节-SPL Token 介绍以及在主网发行代币
tags: [Solana]
---

## 概述

本文全面介绍 Solana 上的 SPL Token 标准，从基础概念到主网部署的完整实践指南。内容涵盖：

- SPL Token 标准及 Token-2022 的新特性
- 铸造账户（Mint）与关联代币账户（ATA）的架构设计
- PDA（程序派生地址）的工作原理与应用场景
- 代币创建、铸造、转账的完整流程
- 主网部署的注意事项与最佳实践
- 空投合约等高级应用的实现

适合有一定 Solana 基础，希望深入理解代币机制并实现代币发行的开发者阅读。

<!--more-->

## SPL Token 简介

### 什么是 SPL Token

SPL Token 是 Solana 上的代币标准，类似于以太坊的 ERC-20。它为开发者提供了统一的代币创建和管理接口，使得钱包、DEX、DeFi 应用都能通用识别和处理代币。

### 核心特点

- **统一标准**：所有 SPL Token 遵循相同的接口规范
- **原生支持**：无需部署合约代码，只需创建铸造账户
- **低成本**：创建代币约需 0.004 SOL（~$0.6）
- **高性能**：得益于 Solana 的高 TPS 和低延迟

### 版本演进

**SPL Token v1**
- 基础功能：铸造、销毁、转账
- 最小可行产品

**SPL Token v2**
- 引入 Token Metadata（由 Metaplex 提供）
- 支持代币名称、符号、图标等信息

**Token-2022 (v3)**
- 更强大的功能：账户冻结、转账钩子等
- 推荐用于新项目
- 本教程采用 Token-2022 标准

### 核心组件

**1. 代币程序**

负责代币的创建、转移、销毁等操作：
- `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb`（Token-2022，推荐）
- `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`（旧版，已弃用）

**2. 铸造账户（Mint Account）**

存储代币的基础数据：
- 总供应量（supply）
- 小数位数（decimals）
- 铸造权限（mint_authority）
- 冻结权限（freeze_authority）
- 扩展数据（如元数据：名称、符号、图标 URI）

**铸造账户数据结构：**

```
┌─────────────────────────────────────┐
│            Mint Account             │
│-------------------------------------│
│  decimals:         u8               │
│  supply:           u64              │
│  mint_authority:   Pubkey           │
│  freeze_authority: Pubkey           │
│-------------------------------------│
│  扩展数据（元数据）                    │
│  - name: "代币名称"                   │
│  - symbol: "代币符号"                 │
│  - uri: "元数据JSON地址"              │
└─────────────────────────────────────┘
```

## 创建 SPL Token

### 基础创建

使用 pxsol 创建代币：

```python
import pxsol

ada = pxsol.wallet.Wallet(pxsol.core.PriKey.int_decode(1))
spl = ada.spl_create(9, {
    'metadata': {
        'name': 'PXSOL',
        'symbol': 'PXS',
        'uri': 'https://raw.githubusercontent.com/mohanson/pxsol/refs/heads/master/res/pxs.json',
    }
})
print(spl)  # 2CMXJX8arHRsiiZadVheRLTd6uhP7DpbaJ9hRiRMSGcF
```

**参数说明：**

- `decimals`: 小数位数（例如 9 表示最小单位为 10⁻⁹）
- `metadata.name`: 代币名称
- `metadata.symbol`: 代币符号
- `metadata.uri`: 元数据 JSON 文件的 URL

**元数据 JSON 文件示例：**

```json
{
    "name": "PXSOL",
    "symbol": "PXS",
    "description": "Proof of study https://github.com/mohanson/pxsol",
    "image": "https://raw.githubusercontent.com/mohanson/pxsol/refs/heads/master/res/pxs.png"
}
```

建议将元数据文件上传到 Arweave 或 IPFS 等永久存储服务。

**费用：** 约 0.004 SOL（租金）+ 0.00001 SOL（交易费）≈ $0.6

## 铸造账户解析

### 查询铸造账户数据

```python
import base64
import pxsol

info = pxsol.rpc.get_account_info('2CMXJX8arHRsiiZadVheRLTd6uhP7DpbaJ9hRiRMSGcF', {})
data = bytearray(base64.b64decode(info['data'][0]))
mint = pxsol.core.TokenMint.serialize_decode(data)
print(mint)
```

**输出示例：**

```json
{
    "auth_mint": "6ASf5EcmmEHTgDJ4X4ZT5vT6iHVJBXPg5AN5YoTCpGWt",
    "supply": 0,
    "decimals": 9,
    "inited": true,
    "auth_freeze": "6ASf5EcmmEHTgDJ4X4ZT5vT6iHVJBXPg5AN5YoTCpGWt",
    "extensions": {
        "metadata_pointer": {...},
        "metadata": [{
            "name": "PXSOL",
            "symbol": "PXS",
            "uri": "https://..."
        }]
    }
}
```

**字段说明：**

- `auth_mint`: 铸造权限账户
- `supply`: 当前总供应量
- `decimals`: 小数位数
- `auth_freeze`: 冻结权限账户
- `extensions`: 扩展数据（元数据指针和元数据）

## 代币操作

### 铸造代币

```python
import pxsol

ada = pxsol.wallet.Wallet(pxsol.core.PriKey.int_decode(1))
spl = pxsol.core.PubKey.base58_decode('2CMXJX8arHRsiiZadVheRLTd6uhP7DpbaJ9hRiRMSGcF')

# 铸造 100,000,000 个代币（考虑 decimals=9）
ada.spl_mint(spl, ada.pubkey, 100000000 * 10 ** 9)
print(ada.spl_balance(spl))  # [100000000000000000, 9]
```

**注意：** 只有拥有铸造权限的账户才能铸造代币。

### 关联代币账户（ATA）

关联代币账户（Associated Token Account）是存储用户代币余额的特殊账户：
- 每个用户和代币组合对应唯一的 ATA
- 地址确定性生成，可预测
- 自动创建（如不存在）

## 深入理解账户架构

在 Solana 的代币系统中，账户之间的关系比较复杂但设计精巧。理解这些关系对于开发代币相关应用至关重要。本节将深入探讨 Mint 账户、ATA、PDA 等核心概念及其相互关系。

### 账户关系详解

下图展示了 Token Mint 账户与多个 Token 账户（ATA）之间的层级关系：

```
┌─────────────────────────────────────────────────────────────┐
│                      Token Mint 账户                         │
│  (代币的"铸造厂"，定义代币种类和规则)                          │
├─────────────────────────────────────────────────────────────┤
│ 地址: pubkey_mint (如 8EzV9...)                              │
│ Owner: Token Program (spl-token-2022)                       │
├─────────────────────────────────────────────────────────────┤
│ 字段:                                                        │
│  - mint_authority: 谁能继续铸造                              │
│  - supply: 总发行量                                          │
│  - decimals: 小数位数 (如 9)                                 │
│  - is_initialized: 是否已初始化                              │
│  - freeze_authority: 谁能冻结账户                            │
└─────────────────────────────────────────────────────────────┘
                         │
                         │ (一个 Mint 可以有多个 Token 账户)
                         │
        ┌────────────────┼────────────────┐
        ▼                ▼                ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│  用户 ATA    │  │  PDA 的 ATA  │  │  其他人 ATA  │
│              │  │              │  │              │
├──────────────┤  ├──────────────┤  ├──────────────┤
│ 地址: 派生自  │  │ 地址: 派生自  │  │ 地址: 派生自  │
│ ATA Program  │  │ ATA Program  │  │ ATA Program  │
│ + user       │  │ + PDA        │  │ + someone    │
│ + mint       │  │ + mint       │  │ + mint       │
├──────────────┤  ├──────────────┤  ├──────────────┤
│ Owner:       │  │ Owner:       │  │ Owner:       │
│ Token-2022   │  │ Token-2022   │  │ Token-2022   │
├──────────────┤  ├──────────────┤  ├──────────────┤
│ 字段:        │  │ 字段:        │  │ 字段:        │
│ - mint       │  │ - mint       │  │ - mint       │
│ - owner      │  │ - owner      │  │ - owner      │
│   (user)     │  │   (PDA)      │  │   (someone)  │
│ - amount     │  │ - amount     │  │ - amount     │
│   余额       │  │   余额       │  │   余额       │
└──────────────┘  └──────────────┘  └──────────────┘
       ▲                 │
       │                 │
       │      转账 5 个代币
       └─────────────────┘
         (由程序 PDA 授权)
```

### ATA 本质上也是 PDA

**重要概念：所有的 ATA 账户本质上都是 PDA（程序派生地址）。**

**1. 用户的 ATA（第一次派生）**

```python
# ATA 地址派生公式
ATA_Address = find_program_address(
    seeds: [
        user_pubkey,           # 用户公钥
        TOKEN_PROGRAM_ID,      # Token 程序 ID
        mint_pubkey            # 代币 Mint 地址
    ],
    program_id: ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID  # ATA 程序
)
```

- ATA 是由 **Associated Token Account Program** 派生的 PDA
- 这确保了每个（用户 + 代币）组合都有唯一的 ATA 地址
- ATA 账户没有私钥，但用户可以通过 Token Program 操作它

**2. PDA 的 ATA（两次派生）**

```python
# 第一次派生：程序创建自己的 PDA
program_pda = find_program_address(
    seeds: [b""],  # 空种子（或其他自定义种子）
    program_id: YOUR_PROGRAM_ID
)

# 第二次派生：为 PDA 创建 ATA
pda_ata = find_program_address(
    seeds: [
        program_pda,           # ← 使用第一次派生的 PDA
        TOKEN_PROGRAM_ID,
        mint_pubkey
    ],
    program_id: ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID
)
```

**核心要点：PDA 的 ATA 经过了两次派生过程。**

### 派生层级对比

```
┌──────────────────────────────────────────────────────────────┐
│ 情况 1：用户的 ATA（派生 1 次）                                │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│  用户公钥 (有私钥)                                            │
│      │                                                        │
│      └─→ ATA Program 派生 ──→ 用户的 ATA (PDA)               │
│           种子: [user, token_program, mint]                   │
│                                                               │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│ 情况 2：PDA 的 ATA（派生 2 次）                                │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│  程序 ID                                                      │
│      │                                                        │
│      └─→ 第一次派生 ──→ 程序 PDA (无私钥)                     │
│           种子: [b""]                                         │
│                  │                                            │
│                  └─→ 第二次派生 ──→ PDA 的 ATA (也是 PDA)    │
│                       种子: [pda, token_program, mint]        │
│                                                               │
└──────────────────────────────────────────────────────────────┘
```

**关键区别：**

| 特性 | 用户的 ATA | PDA 的 ATA |
|------|-----------|-----------|
| 派生次数 | 1 次 | 2 次 |
| ATA 的 owner | 用户公钥（有私钥） | 程序 PDA（无私钥） |
| 谁能控制 | 用户通过私钥签名 | 程序通过 `invoke_signed` |
| 派生参与者 | ATA Program | 你的程序 + ATA Program |
| 账户类型 | 都是 PDA | 都是 PDA |

**两者共同点：**
- ATA 账户本身都是由 ATA Program 派生的 PDA
- ATA 账户的 program owner 都是 Token Program
- ATA 账户都存储 `mint`、`owner`、`amount` 等字段

### 理解 PDA 和 PDA 的 ATA

**PDA（Program Derived Address）** 是程序派生地址，它是一个特殊的账户：
- **无私钥**：PDA 没有对应的私钥，无法直接签名交易
- **程序控制**：只能由创建它的程序通过 `invoke_signed` 来控制
- **确定性派生**：使用程序 ID + 种子（seeds）+ bump 派生而来

**PDA 的 ATA** 就是：**以 PDA 作为 owner 的关联代币账户**（经过两次 PDA 派生）

### PDA 派生机制

PDA 的派生**不需要钱包地址**，而是通过以下方式：

```python
# 示例：派生程序的 PDA
pubkey_mana = pxsol.core.PubKey.base58_decode('HgatfFyGw2bLJeTy9HkVd4ESD6FkKu4TqMYgALsWZnE6')  # 程序地址
pubkey_mana_auth = pubkey_mana.derive_pda(bytearray([]))[0]  # 使用空种子派生 PDA
```

**派生公式：**

```
PDA = find_program_address(
    seeds: [seed1, seed2, ...],  // 任意字节数组，可以为空
    program_id: Pubkey           // 程序 ID
)
```

**常见种子选择：**
- **空种子** `[]`：最简单的方式，一个程序只有一个这样的 PDA
- **字符串** `[b"vault"]`：语义化的种子
- **用户地址** `[user_pubkey]`：为每个用户创建独立的 PDA
- **组合种子** `[b"vault", user_pubkey, mint_pubkey]`：更复杂的场景

### 空投合约中的账户层级示例

为了更好地理解 PDA 的实际应用，下面以空投合约为例说明账户层级关系：

```
程序账户 (HgatfFy...)
    │
    ├─ 派生 PDA (使用空种子 [])
    │   │
    │   └─ PDA 的 ATA (存储 90,000,000 PXS)
    │       └─ Owner: PDA
    │       └─ Mint: 代币地址
    │
    └─ 程序逻辑
        └─ 使用 invoke_signed 让 PDA 签名
        └─ 从 PDA 的 ATA 转账给用户
```

**关键点：**
1. 程序部署后有固定地址（如 `HgatfFy...`）
2. 程序使用空种子 `[]` 派生出唯一的 PDA
3. 为这个 PDA 创建 ATA，用于存储代币
4. 用户调用程序时，程序使用 `invoke_signed(&[&[seed, &[bump]]])` 让 PDA "签名"
5. PDA 的 ATA 自动转账给用户的 ATA

### 实际代码示例

下面通过代码演示用户 ATA 和 PDA ATA 的派生过程：

```python
import pxsol

# === 情况 1：用户的 ATA（1 次派生）===
user_pubkey = pxsol.core.PubKey.base58_decode('6ASf5Ec...')  # 用户地址（有私钥）
mint_pubkey = pxsol.core.PubKey.base58_decode('2CMXJX8...')  # 代币 Mint

# ATA Program 自动派生用户的 ATA
user_ata = pxsol.core.spl_token.get_associated_token_address(
    user_pubkey,  # owner
    mint_pubkey   # mint
)
# 结果：用户的 ATA 地址（由 ATA Program 派生的 PDA）

# === 情况 2：PDA 的 ATA（2 次派生）===
program_id = pxsol.core.PubKey.base58_decode('HgatfFy...')  # 程序地址

# 第 1 次派生：程序创建自己的 PDA
program_pda, bump = program_id.derive_pda(bytearray([]))  # 空种子
# 结果：program_pda 是一个 PDA（无私钥）

# 第 2 次派生：为 PDA 创建 ATA
pda_ata = pxsol.core.spl_token.get_associated_token_address(
    program_pda,  # owner（注意：这里是 PDA，不是普通公钥）
    mint_pubkey   # mint
)
# 结果：PDA 的 ATA 地址（也是由 ATA Program 派生的 PDA）

# 程序如何转账？
# 程序需要用 bump 和种子来"证明"自己控制这个 PDA
invoke_signed(
    transfer_instruction,
    accounts,
    &[&[b"", &[bump]]]  # 签名者种子：空种子 + bump
)
```

### 为什么需要 PDA？

PDA 在 Solana 程序开发中扮演着重要角色：

1. **程序无法直接持有资产**：程序账户是可执行的，不能作为代币 owner
2. **安全的资产托管**：PDA 作为"程序控制的钱包"，既能存储代币，又能由程序控制转账
3. **无私钥设计**：确保只有程序能操作，避免私钥泄露风险
4. **确定性地址**：两次派生机制保证了地址的唯一性和可预测性
5. **灵活的权限管理**：通过不同的种子可以为不同场景创建独立的 PDA

## 代币操作实战

### 转账

```python
import pxsol

ada = pxsol.wallet.Wallet(pxsol.core.PriKey.int_decode(1))
bob = pxsol.wallet.Wallet(pxsol.core.PriKey.int_decode(2))
spl = pxsol.core.PubKey.base58_decode('2CMXJX8arHRsiiZadVheRLTd6uhP7DpbaJ9hRiRMSGcF')

# 转账 100 个代币给 Bob
print(ada.spl_balance(spl))  # [100000000000000000, 9]
ada.spl_transfer(spl, bob.pubkey, 100 * 10 ** 9)
print(ada.spl_balance(spl))  # [99999900000000000, 9]
print(bob.spl_balance(spl))  # [100000000000, 9]
```

## 指令详解

### 创建代币指令 (spl_create)
`spl_create()` 方法包含 4 条链上指令：

**指令 1：创建铸造账户**
- 分配租金豁免的账户空间
- 设置账户所有者为 Token 程序

**指令 2：初始化元数据指针**
- 启用 Token-2022 的 metadata pointer 扩展
- 指定元数据存储位置

**指令 3：初始化铸造账户**
- 设置 decimals（小数位数）
- 设置 mint_authority（铸造权限）
- 设置 freeze_authority（冻结权限）

**指令 4：初始化元数据**
- 写入代币名称、符号、URI

### 铸造代币指令 (spl_mint)

`spl_mint()` 方法包含 2 条链上指令：

**指令 1：创建关联代币账户**
- 使用 `create_idempotent()` 确保幂等性
- 如果账户已存在则跳过

**指令 2：铸造代币**
- 调用 `mint_to()` 增加代币供应
- 需要铸造权限

### 转账指令 (spl_transfer)

`spl_transfer()` 方法包含 2 条链上指令：

**指令 1：创建接收者的关联代币账户**
- 同 `spl_mint()` 的第一条指令

**指令 2：执行转账**
- 调用 `transfer()` 转移代币
- 不改变总供应量

## 主网部署

### 环境切换
**网络对比：**

| 网络 | 特点 | 用途 |
|------|------|------|
| Localnet | 本地节点，速度最快 | 开发测试 |
| Devnet | 公共测试网 | 合约测试 |
| Mainnet Beta | 主网 | 生产环境 |

**切换环境：**

```python
import pxsol

pxsol.config.current = pxsol.config.develop  # Localnet（默认）
pxsol.config.current = pxsol.config.testnet  # Devnet
pxsol.config.current = pxsol.config.mainnet  # Mainnet
```

### 主网注意事项

**费用预算：**
- 创建代币：~0.004 SOL
- 部署空投合约：~0.5 SOL
- 创建流动性池：0.01~0.05 SOL
- **建议预算：** 至少 1 SOL

**RPC 限速：**

```python
pxsol.config.current = pxsol.config.mainnet
pxsol.config.current.rpc.qps = 1
pxsol.config.current.rpc.url = 'https://api.mainnet-beta.solana.com'
```

推荐使用付费 RPC 服务（Helius、Triton One）以获得更好的性能。

**元数据托管：**
- 推荐：Arweave 或 IPFS（永久存储）
- 备选：GitHub（可能不稳定）

### 在主网发行代币

```python
import pxsol

pxsol.config.current = pxsol.config.mainnet

you = pxsol.wallet.Wallet(pxsol.core.PriKey.base58_decode('YOUR_PRIVATE_KEY'))
spl = you.spl_create(9, {
    'metadata': {
        'name': 'PXSOL',
        'symbol': 'PXS',
        'uri': 'https://raw.githubusercontent.com/mohanson/pxsol/refs/heads/master/res/pxs.json',
    }
})
print(spl)  # 代币地址

# 铸造初始代币
you.spl_mint(spl, you.pubkey, 100000000 * 10 ** 9)
```

### 上架 DEX

以 Raydium 为例：

1. 访问 https://raydium.io/liquidity-pools/
2. 点击 Create 创建流动性池
3. 选择代币对（如 PXS/SOL）
4. 设置初始价格和交易费率
5. 提供流动性并获得 LP Token

**注意：** 妥善保管 LP Token，撤出流动性时需要。

## 高级应用：空投合约

### 程序控制的代币

要实现自动空投，需要理解程序如何控制代币：

**账户层级：**
1. 程序账户：空投合约本身
2. 程序 PDA 账户：代币实际持有者
3. 关联代币账户：存储代币余额

**核心机制：**
- PDA 账户无私钥，由程序通过 `invoke_signed` 控制
- 程序使用种子（seed）和 bump 签署交易

### 实现空投程序
**功能：**
1. 自动为调用者创建关联代币账户
2. 转账 5 PXS 给调用者

**Rust 实现示例（简化版）：**

```rust
use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    program::invoke_signed,
    pubkey::Pubkey,
};

entrypoint!(process_instruction);

fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    // 1. 创建用户的关联代币账户（幂等）
    solana_program::program::invoke(
        &spl_associated_token_account::instruction::create_associated_token_account_idempotent(...),
        accounts,
    )?;
    
    // 2. 使用 invoke_signed 从 PDA 转账 5 PXS
    solana_program::program::invoke_signed(
        &spl_token_2022::instruction::transfer_checked(
            ...,
            5_000_000_000,  // 5 PXS（decimals=9）
            9,
        )?,
        accounts,
        &[&[seed, &[bump]]],  // PDA 签名种子
    )?;
    
    Ok(())
}
```

**部署合约：**

```python
import pxsol

pxsol.config.current = pxsol.config.mainnet

user = pxsol.wallet.Wallet(pxsol.core.PriKey.base58_decode('YOUR_PRIVATE_KEY'))
with open('target/deploy/pxsol_spl.so', 'rb') as f:
    data = bytearray(f.read())
mana = user.program_deploy(data)
print(mana)  # 合约地址
```

**为合约充值代币：**

```python
# 转入 90,000,000 PXS 到程序 PDA 账户
pubkey_mint = pxsol.core.PubKey.base58_decode('6B1ztFd9wSm3J5zD5vmMNEKg2r85M41wZMUW7wXwvEPH')
pubkey_mana = pxsol.core.PubKey.base58_decode('HgatfFyGw2bLJeTy9HkVd4ESD6FkKu4TqMYgALsWZnE6')
pubkey_mana_auth = pubkey_mana.derive_pda(bytearray([]))[0]
user.spl_transfer(pubkey_mint, pubkey_mana_auth, 90000000 * 10**9)
```

### 领取空投

**运行空投脚本：**

```bash
$ git clone https://github.com/mohanson/pxsol
$ cd pxsol
$ python example/pxs_airdrop.py --prikey YOUR_PRIVATE_KEY
```

## 示例项目

完整源码：https://github.com/mohanson/pxsol-spl

**本地测试：**

```bash
$ git clone https://github.com/mohanson/pxsol-spl
$ cd pxsol-spl

# 部署合约
$ python make.py deploy

# 生成测试账户
$ python make.py genuser

# 领取空投
$ python make.py --prikey YOUR_PRIVATE_KEY airdrop
```

## 总结

本文全面介绍了 Solana SPL Token 的核心概念和完整开发流程：

### 核心知识点

1. **基础概念**
   - SPL Token 标准及其演进历程
   - 铸造账户（Mint Account）的数据结构
   - 关联代币账户（ATA）的工作原理

2. **账户架构**
   - Token Mint 与 Token Account 的关系
   - ATA 作为 PDA 的本质
   - 用户 ATA（单次派生）vs PDA 的 ATA（两次派生）
   - PDA 派生机制与种子选择策略

3. **代币操作**
   - 使用 Token-2022 创建带元数据的代币
   - 铸造、转账、查询余额等基础操作
   - 底层指令的组成与执行流程

4. **主网部署**
   - 开发环境、测试网、主网的切换
   - 费用预算与 RPC 配置优化
   - 元数据托管的最佳实践

5. **高级应用**
   - DEX 流动性池创建
   - 空投合约的设计与实现
   - 程序通过 PDA 控制代币的机制

通过实践这些步骤，您可以完整掌握 Solana 代币开发的全流程，从本地测试到主网发布，从基础操作到高级应用。

## 参考资料

### 官方文档
- [Solana 主网项目部署指南](https://accu.cc/content/solana/project_mainnet/)

### 示例代码
- [Solana Program 实现（pxsol-spl）](https://github.com/mohanson/pxsol-spl) - 使用原生 Solana Program 开发的空投合约
- [Anchor 框架实现（pxsol-spl-anchor）](https://github.com/mohanson/pxsol-spl-anchor) - 使用 Anchor 框架的实现版本
