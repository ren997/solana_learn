---
title: 第七节 - Pinocchio 入门
tags: [Solana]
---

# Pinocchio 入门

> 作者：@Blueshift

## 目录

- [Pinocchio 简介](#pinocchio-简介)
- [原生开发](#原生开发)
- [入口点 (Entrypoint)](#入口点-entrypoint)
- [账户和指令](#账户和指令)
- [账户类型实现](#账户类型实现)
- [指令结构](#指令结构)
- [跨程序调用 (CPI)](#跨程序调用-cpi)
- [错误处理](#错误处理)
- [数据读写](#数据读写)
- [性能优化](#性能优化)
- [测试](#测试)
- [总结](#总结)

---

## Pinocchio 简介

### 什么是 Pinocchio

虽然大多数 Solana 开发者依赖 Anchor 框架,但有很多充分的理由选择不使用它编写程序:

- 需要对每个账户字段进行更精细的控制
- 追求极致的性能
- 想要避免使用宏

在没有像 Anchor 这样的框架支持下编写 Solana 程序被称为**原生开发**。这更具挑战性,但在本课程中,您将学习如何使用 Pinocchio 从零开始构建一个 Solana 程序。

**Pinocchio** 是一个极简的 Rust 库,它允许您在不引入重量级 `solana-program` crate 的情况下编写 Solana 程序。它通过将传入的交易负载(账户、指令数据等所有内容)视为单个字节切片,并通过零拷贝技术就地读取。

### 主要优势

极简设计带来了三大优势:

1. **更少的计算单元** - 没有额外的反序列化或内存拷贝
2. **更小的二进制文件** - 更精简的代码路径意味着更轻量的 `.so` 链上程序
3. **零依赖拖累** - 没有需要更新(或可能破坏)的外部 crate

该项目由 Febo 在 Anza 发起,并得到了 Solana 生态系统和 Blueshift 团队的核心贡献。

除了核心 crate,您还会发现 `pinocchio-system` 和 `pinocchio-token`,它们为 Solana 的原生 System 和 SPL-Token 程序提供了零拷贝辅助工具和 CPI 实用程序。

---

## 原生开发

原生开发可能听起来令人望而生畏,但这正是本章节存在的原因。在本章节结束时,您将了解跨越程序边界的每一个字节,以及如何保持您的逻辑紧凑、安全和高效。

Anchor 使用**过程宏和派生宏**来简化处理账户、instruction data 和错误处理的样板代码,这些是构建 Solana 程序的核心。

### 原生开发的要求

原生开发意味着我们不再享有这种便利,我们需要:

- 为不同的指令创建我们自己的 Discriminator 和 Entrypoint
- 创建我们自己的账户、指令和反序列化逻辑
- 实现所有 Anchor 之前为我们处理的安全检查

> **注意**: 目前还没有用于构建 Pinocchio 程序的"框架"。因此,我们将基于我们的经验,介绍我们认为是编写 Pinocchio 程序的最佳方法。

---

## 入口点 (Entrypoint)

### Anchor vs Pinocchio

在 Anchor 中,`#[program]` 宏隐藏了许多底层逻辑。它在底层为每个指令和账户构建了一个 8 字节的 Discriminator(从 0.31 版本开始支持自定义大小)。

**Anchor Discriminator 计算方式:**

| 类型 | 计算方法 | 示例 |
|------|----------|------|
| Account | `sha256("account:" + PascalCase(seed))[0..8]` | `[21, 124, 154, 78, 247, 222, 89, 189]` |
| Instruction | `sha256("global:" + snake_case(seed))[0..8]` | `[163, 36, 134, 53, 232, 223, 146, 222]` |

原生程序通常更加精简。单字节的 Discriminator(值范围为 `0x01…0xFF`)足以支持最多 255 个指令,这对于大多数用例来说已经足够。如果需要更多,可以切换到双字节变体,扩展到 65,535 种可能的变体。

### entrypoint! 宏

`entrypoint!` 宏是程序执行的起点。它提供了三个原始切片:

- `program_id`: 已部署程序的公钥
- `accounts`: 指令中传递的所有账户
- `instruction_data`: 包含 Discriminator 和用户提供数据的不透明字节数组

### 典型的入口点实现

```rust
entrypoint!(process_instruction);

fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match instruction_data.split_first() {
        Some((Instruction1::DISCRIMINATOR, data)) => {
            Instruction1::try_from((data, accounts))?.process()
        }
        Some((Instruction2::DISCRIMINATOR, _)) => {
            Instruction2::try_from(accounts)?.process()
        }
        _ => Err(ProgramError::InvalidInstructionData)
    }
}
```

### 处理器工作流程

在幕后,这个处理器:

1. 使用 `split_first()` 提取判别字节
2. 使用 `match` 确定要实例化的指令结构
3. 每个指令的 `try_from` 实现会验证并反序列化其输入
4. 调用 `process()` 执行业务逻辑

### solana-program 和 pinocchio 的区别

主要的区别和优化在于 `entrypoint()` 的行为方式:

**标准 Solana 入口点:**
- 使用传统的序列化模式
- 运行时会预先反序列化输入数据
- 在内存中创建拥有的数据结构
- 广泛使用 Borsh 序列化
- 在反序列化过程中复制数据

**Pinocchio 入口点:**
- 直接从输入字节数组中读取数据而不进行复制
- 实现零拷贝操作
- 定义了引用原始数据的零拷贝类型
- 消除了序列化/反序列化的开销
- 通过直接内存访问避免了抽象层

---

## 账户和指令

由于我们没有宏,并且为了保持程序的精简和高效,因此每个指令数据字节和账户都必须手动验证。

为了使这个过程更有条理,我们使用了一种模式,该模式提供了类似 Anchor 的易用性,但没有使用宏,从而通过实现 Rust 的 `TryFrom` trait,使实际的 `process()` 方法几乎没有样板代码。

### TryFrom Trait

`TryFrom` 是 Rust 标准转换家族的一部分。与 `From` 假设转换不会失败不同,`TryFrom` 返回一个 `Result`,允许您及早暴露错误——非常适合链上验证。

```rust
pub trait TryFrom<T>: Sized {
    type Error;
    fn try_from(value: T) -> Result<Self, Self::Error>;
}
```

在 Solana 程序中,我们实现 `TryFrom` 来将原始账户切片(以及在需要时的指令字节)转换为强类型结构,同时强制执行每个约束。

#### 💡 与 Anchor 的对比

**`TryFrom` 在 Pinocchio 中就是用来替代 Anchor 的 `#[derive(Accounts)]` 宏!**

**Anchor 方式 (自动):**
```rust
#[derive(Accounts)]
pub struct Transfer<'info> {
    #[account(mut)]
    pub from: Signer<'info>,

    #[account(mut)]
    pub to: SystemAccount<'info>,
}
// ✅ Anchor 宏自动生成所有验证代码
```

**Pinocchio 方式 (手动):**
```rust
pub struct TransferAccounts<'a> {
    pub from: &'a AccountInfo,
    pub to: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for TransferAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [from, to] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // 手动实现 Anchor 宏自动做的检查
        if !from.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !from.is_writable || !to.is_writable {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self { from, to })
    }
}
// ✅ 你手动编写所有验证逻辑,完全掌控
```

**核心区别:**
- **Anchor**: 宏自动生成验证代码 → 简单但有性能开销
- **Pinocchio**: 手动实现 `TryFrom` → 更多代码但零开销,完全控制

### 账户验证

我们通常在每个 `TryFrom` 实现中处理所有不需要双重借用的特定检查。这使得所有指令逻辑发生的 `process()` 函数尽可能简洁。

我们从实现指令所需的账户结构开始,类似于 Anchor 的 `Context`。

> **注意**: 与 Anchor 不同,在这个账户结构中,我们只包括在处理过程中需要使用的账户,并将指令中需要但不会使用的其余账户(例如 SystemProgram)标记为 `_`。

#### 示例: Deposit 账户结构

```rust
pub struct DepositAccounts<'a> {
    pub owner: &'a AccountInfo,
    pub vault: &'a AccountInfo,
}
```

#### 实现 TryFrom 进行验证

```rust
impl<'a> TryFrom<&'a [AccountInfo]> for DepositAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // 1. 解构切片
        let [owner, vault, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // 2. 自定义检查
        if !owner.is_signer() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if !vault.is_owned_by(&pinocchio_system::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // 3. 返回验证后的结构
        Ok(Self { owner, vault })
    }
}
```

### 指令验证

指令验证遵循与账户验证类似的模式。

#### 定义指令数据结构

```rust
pub struct DepositInstructionData {
    pub amount: u64,
}
```

#### 实现 TryFrom 验证

```rust
impl<'a> TryFrom<&'a [u8]> for DepositInstructionData {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        // 1. 验证数据长度
        if data.len() != core::mem::size_of::<u64>() {
            return Err(ProgramError::InvalidInstructionData);
        }

        // 2. 转换字节切片为 u64
        let amount = u64::from_le_bytes(data.try_into().unwrap());

        // 3. 验证金额(例如,确保不为零)
        if amount == 0 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self { amount })
    }
}
```

### 这种模式的优势

- 在 instruction data 进入业务逻辑之前进行验证
- 将验证逻辑与核心功能分离
- 在验证失败时提供清晰的错误信息
- 在整个程序中保持类型安全性

---

## 账户类型实现

正如我们在上一节中看到的,与 Anchor 不同,Pinocchio 的账户验证无法使用自动执行所有者、签名和标识符检查的账户类型。

在原生 Rust 中,我们需要手动执行这些验证。

### 基本验证示例

```rust
// SignerAccount 类型检查
if !account.is_signer() {
    return Err(PinocchioError::NotSigner.into());
}

// SystemAccount 类型检查
if !account.is_owned_by(&pinocchio_system::ID) {
    return Err(PinocchioError::InvalidOwner.into());
}
```

通过将所有验证封装在 `TryFrom` 实现中,我们可以轻松识别缺失的检查并确保我们编写的是安全的代码。

然而,为每个指令编写这些检查可能会变得重复。为了解决这个问题,我们创建了一个 `helper.rs` 文件,该文件定义了类似于 Anchor 的类型,以简化这些验证。

### 通用接口和特性 (Traits)

对于我们的 `helper.rs` 文件,我们利用了 Rust 的两个基本概念:**通用接口**和**特性**。

#### 为什么选择 Traits 而不是宏?

我们选择这种方法而不是基于宏的解决方案有几个关键原因:

1. **清晰明确** - 特性和接口提供了清晰、明确的代码,读者无需在脑海中"展开"宏即可理解
2. **编译器验证** - 编译器可以验证特性实现,从而实现更好的错误检测、类型推断、自动补全和重构工具
3. **代码重用** - 特性允许通用实现,可以重复使用而无需代码重复,而过程宏会为每次使用生成重复代码
4. **可打包性** - 这些特性可以打包成可重用的 crate,而宏生成的 API 通常仅限于定义它们的 crate

### 什么是 Traits 和通用接口?

如果您熟悉其他编程语言,您可能会发现 traits 类似于"接口";它们定义了一个契约,规定了某个类型必须实现哪些方法。

在 Rust 中,trait 充当一个蓝图,声明"任何实现此 trait 的类型必须提供这些特定的函数"。

#### 简单示例

```rust
// 定义 Trait
pub trait AccountCheck {
    fn check(account: &AccountInfo) -> Result<(), ProgramError>;
}

// 定义类型
pub struct SignerAccount;

// 为不同类型实现 trait
impl AccountCheck for SignerAccount {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if !account.is_signer() {
            return Err(PinocchioError::NotSigner.into());
        }
        Ok(())
    }
}

pub struct SystemAccount;

impl AccountCheck for SystemAccount {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if !account.is_owned_by(&pinocchio_system::ID) {
            return Err(PinocchioError::InvalidOwner.into());
        }
        Ok(())
    }
}
```

这里的妙处在于,任何实现了 `AccountCheck` 的账户类型都可以以相同的方式使用;我们可以对它们中的任何一个调用 `.check()`,并且每种类型都处理适合其自身的验证逻辑。

这就是我们所说的"通用接口":不同的类型共享相同的方法签名。

---

## 签名者和系统账户

正如我们在之前的示例中看到的,`SystemAccount` 和 `SignerAccount` 检查非常简单,不需要任何额外的验证。

```rust
pub trait AccountCheck {
    fn check(account: &AccountInfo) -> Result<(), ProgramError>;
}

pub struct SignerAccount;

impl AccountCheck for SignerAccount {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if !account.is_signer() {
            return Err(PinocchioError::NotSigner.into());
        }
        Ok(())
    }
}

pub struct SystemAccount;

impl AccountCheck for SystemAccount {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if !account.is_owned_by(&pinocchio_system::ID) {
            return Err(PinocchioError::InvalidOwner.into());
        }
        Ok(())
    }
}
```

这里我们只是检查账户是否是签名者,或者是否由系统程序拥有。请注意,这两个结构体都提供了相同的检查方法,为我们提供了前面提到的通用接口。

---

## 铸币账户和代币账户

现在事情变得更有趣了。我们从常规的 `AccountCheck` trait 开始,但我们还添加了其他特定的 traits,以提供类似于 Anchor 宏的额外辅助功能,例如 `init` 和 `init_if_needed`。

### MintAccount 实现

```rust
pub struct MintAccount;

impl AccountCheck for MintAccount {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if !account.is_owned_by(&pinocchio_token::ID) {
            return Err(PinocchioError::InvalidOwner.into());
        }

        if account.data_len() != pinocchio_token::state::Mint::LEN {
            return Err(PinocchioError::InvalidAccountData.into());
        }

        Ok(())
    }
}
```

### MintInit Trait

对于 `init` 和 `init_if_needed` 的功能,我们创建了另一个名为 `MintInit` 的 trait:

```rust
pub trait MintInit {
    fn init(
        account: &AccountInfo,
        payer: &AccountInfo,
        decimals: u8,
        mint_authority: &[u8; 32],
        freeze_authority: Option<&[u8; 32]>
    ) -> ProgramResult;

    fn init_if_needed(
        account: &AccountInfo,
        payer: &AccountInfo,
        decimals: u8,
        mint_authority: &[u8; 32],
        freeze_authority: Option<&[u8; 32]>
    ) -> ProgramResult;
}
```

### MintInit 实现

```rust
impl MintInit for MintAccount {
    fn init(
        account: &AccountInfo,
        payer: &AccountInfo,
        decimals: u8,
        mint_authority: &[u8; 32],
        freeze_authority: Option<&[u8; 32]>
    ) -> ProgramResult {
        // 获取租金所需的 lamports
        let lamports = Rent::get()?.minimum_balance(pinocchio_token::state::Mint::LEN);

        // 为账户提供所需的 lamports
        CreateAccount {
            from: payer,
            to: account,
            lamports,
            space: pinocchio_token::state::Mint::LEN as u64,
            owner: &pinocchio_token::ID,
        }.invoke()?;

        InitializeMint2 {
            mint: account,
            decimals,
            mint_authority,
            freeze_authority,
        }.invoke()
    }

    fn init_if_needed(
        account: &AccountInfo,
        payer: &AccountInfo,
        decimals: u8,
        mint_authority: &[u8; 32],
        freeze_authority: Option<&[u8; 32]>
    ) -> ProgramResult {
        match Self::check(account) {
            Ok(_) => Ok(()),
            Err(_) => Self::init(account, payer, decimals, mint_authority, freeze_authority),
        }
    }
}
```

### TokenAccount 实现

然后我们对 `TokenAccount` 执行完全相同的操作:

```rust
pub struct TokenAccount;

impl AccountCheck for TokenAccount {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if !account.is_owned_by(&pinocchio_token::ID) {
            return Err(PinocchioError::InvalidOwner.into());
        }

        if account.data_len().ne(&pinocchio_token::state::TokenAccount::LEN) {
            return Err(PinocchioError::InvalidAccountData.into());
        }

        Ok(())
    }
}
```

---

## Token2022 支持

对于传统的 SPL Token Program,我们仅对 Mint 和 TokenAccount 进行了长度检查。这种方法之所以有效,是因为当您只有两种固定大小的账户类型时,可以仅通过它们的长度来区分它们。

对于 Token2022,这种简单的方法不起作用。当直接将 token extensions 添加到 Mint 数据时,其大小可能会增长并可能超过 TokenAccount 的大小。

### Token2022 区分方式

对于 Token2022,我们可以通过两种方式区分 Mint 和 TokenAccount:

1. **通过大小** - 类似于传统的 Token Program(当账户具有标准大小时)
2. **通过 discriminator** - 一个位于位置 165 的特殊字节(比传统的 TokenAccount 大一个字节,以避免冲突)

### Token2022 常量定义

```rust
// TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb
pub const TOKEN_2022_PROGRAM_ID: [u8; 32] = [
    0x06, 0xdd, 0xf6, 0xe1, 0xee, 0x75, 0x8f, 0xde,
    0x18, 0x42, 0x5d, 0xbc, 0xe4, 0x6c, 0xcd, 0xda,
    0xb6, 0x1a, 0xfc, 0x4d, 0x83, 0xb9, 0x0d, 0x27,
    0xfe, 0xbd, 0xf9, 0x28, 0xd8, 0xa1, 0x8b, 0xfc,
];

const TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET: usize = 165;
pub const TOKEN_2022_MINT_DISCRIMINATOR: u8 = 0x01;
pub const TOKEN_2022_TOKEN_ACCOUNT_DISCRIMINATOR: u8 = 0x02;
```

### Mint2022Account 实现

```rust
pub struct Mint2022Account;

impl AccountCheck for Mint2022Account {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if !account.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            return Err(PinocchioError::InvalidOwner.into());
        }

        let data = account.try_borrow_data()?;

        if data.len().ne(&pinocchio_token::state::Mint::LEN) {
            if data.len().le(&TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET) {
                return Err(PinocchioError::InvalidAccountData.into());
            }
            if data[TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET]
                .ne(&TOKEN_2022_MINT_DISCRIMINATOR) {
                return Err(PinocchioError::InvalidAccountData.into());
            }
        }

        Ok(())
    }
}
```

---

## 指令结构

正如我们之前所看到的,使用 `TryFrom` trait 可以将验证与业务逻辑清晰地分离,从而提高可维护性和安全性。

### 定义指令结构

当需要处理逻辑时,我们可以创建如下结构:

```rust
pub struct Deposit<'a> {
    pub accounts: DepositAccounts<'a>,
    pub instruction_datas: DepositInstructionData,
}
```

此结构定义了在逻辑处理期间可访问的数据。

### 实现 TryFrom

```rust
impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for Deposit<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo]))
        -> Result<Self, Self::Error> {
        let accounts = DepositAccounts::try_from(accounts)?;
        let instruction_datas = DepositInstructionData::try_from(data)?;

        Ok(Self {
            accounts,
            instruction_datas,
        })
    }
}
```

### 包装器的优势

此包装器提供了三个关键优势:

1. 它接受原始输入(字节和账户)
2. 它将验证委托给各个 `TryFrom` 实现
3. 它返回一个完全类型化、完全验证的 `Deposit` 结构

### 实现处理逻辑

```rust
impl<'a> Deposit<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;

    pub fn process(&self) -> ProgramResult {
        // deposit 逻辑
        Ok(())
    }
}
```

- `DISCRIMINATOR` 是我们在入口点中用于模式匹配的字节
- `process()` 方法仅包含业务逻辑,因为所有验证检查都已完成

**结果**: 我们获得了 Anchor 风格的易用性,同时具备完全原生的所有优势:明确、可预测且快速。

---

## 跨程序调用 (CPI)

如前所述,Pinocchio 提供了像 `pinocchio-system` 和 `pinocchio-token` 这样的辅助 crate,简化了对原生程序的跨程序调用(CPI)。

### 简单的 CPI 调用

这些辅助结构和方法取代了我们之前使用的 Anchor 的 `CpiContext` 方法:

```rust
Transfer {
    from: self.accounts.owner,
    to: self.accounts.vault,
    lamports: self.instruction_datas.amount,
}
.invoke()?;
```

`Transfer` 结构(来自 `pinocchio-system`)封装了 System Program 所需的所有字段,而 `.invoke()` 执行了 CPI。无需上下文构建器或额外的样板代码。

### 带签名的 CPI (PDA)

当调用者必须是一个程序派生地址(PDA)时,Pinocchio 保持了同样简洁的 API:

```rust
let seeds = [
    Seed::from(b"vault"),
    Seed::from(self.accounts.owner.key().as_ref()),
    Seed::from(&[bump]),
];
let signers = [Signer::from(&seeds)];

Transfer {
    from: self.accounts.vault,
    to: self.accounts.owner,
    lamports: self.accounts.vault.lamports(),
}
.invoke_signed(&signers)?;
```

### 操作方式

1. `Seeds` 创建一个与 PDA 派生相匹配的 `Seed` 对象数组
2. `Signer` 将这些种子封装在一个 `Signer` 辅助工具中
3. `invoke_signed` 执行 CPI,传递签名者数组以授权转账

**结果**: 一个干净的、一流的接口,适用于常规和签名的 CPI:无需宏,也没有隐藏的魔法。

---

## 错误处理

清晰且描述性强的错误类型对于使用 Pinocchio 构建的 Solana 程序至关重要。它们可以让调试更容易,并为与您的程序交互的用户和客户端提供有意义的反馈。

### 为什么选择 thiserror

在 Rust 中定义自定义错误类型时,您有多种选择,例如 `thiserror`、`anyhow` 和 `failure`。对于 Pinocchio 程序,`thiserror` 是首选,因为:

1. 它允许您使用 `#[error("...")]` 属性为每个错误变体添加可读的消息注释
2. 它会自动实现 `core::error::Error` 和 `Display` 特性,使您的错误易于打印和调试
3. 所有错误消息和格式在编译时检查,降低了运行时问题的风险
4. 最重要的是,`thiserror` 支持在禁用其默认功能时的 `no_std` 环境,这是 Pinocchio 程序的必要条件

### 添加依赖

在 `Cargo.toml` 中添加:

```toml
[dependencies]
thiserror = { version = "2.0", default-features = false }
```

### 定义错误枚举

```rust
use {
    num_derive::FromPrimitive,
    pinocchio::program_error::{ProgramError, ToStr},
    thiserror::Error,
};

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum PinocchioError {
    // 0
    /// Lamport 余额低于免租金阈值
    #[error("Lamport balance below rent-exempt threshold")]
    NotRentExempt,
}
```

### 实现 From<PinocchioError> for ProgramError

```rust
impl From<PinocchioError> for ProgramError {
    fn from(e: PinocchioError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
```

这使您可以使用 `?` 操作符并无缝返回您的自定义错误。

### 从原始值反序列化错误(可选)

```rust
impl TryFrom<u32> for PinocchioError {
    type Error = ProgramError;

    fn try_from(error: u32) -> Result<Self, Self::Error> {
        match error {
            0 => Ok(PinocchioError::NotRentExempt),
            _ => Err(ProgramError::InvalidArgument),
        }
    }
}
```

### 可读性强的错误信息(可选)

```rust
impl ToStr for PinocchioError {
    fn to_str<E>(&self) -> &'static str {
        match self {
            PinocchioError::NotRentExempt =>
                "Error: Lamport balance below rent-exempt threshold",
        }
    }
}
```

---

## 性能优化

虽然许多开发者选择 Pinocchio 是因为它对账户字段的精细控制,但它的真正优势在于实现最大性能。

### 冗余检查

开发者通常会为了安全性添加额外的账户约束,但这些可能会引入不必要的开销。区分必要检查和冗余检查非常重要。

**示例**: 当仅从 Token Account 或 Mint 读取数据时,反序列化和验证是必要的。但如果这些相同的账户随后用于 CPI(跨程序调用),任何不匹配或错误都会导致指令在该点失败。因此,预先检查可能是多余的。

同样,验证 Token Account 的"所有者"通常是多余的;特别是当账户由 PDA(程序派生地址)控制时。如果所有者不正确,CPI 将因无效的种子而失败。

### 关联 Token Program

Associated Token Accounts(ATA)很方便,但会带来性能成本。除非绝对必要,否则避免强制使用它们,并且永远不要在指令逻辑中要求创建它们。

如果您的程序依赖于 ATA,请确保它们在外部创建。在您的程序中,通过直接派生预期地址来验证其正确性:

```rust
let (associated_token_account, _) = find_program_address(
    &[
        self.accounts.owner.key(),
        self.accounts.token_program.key(),
        self.accounts.mint.key(),
    ],
    &pinocchio_associated_token_account::ID,
);
```

### 性能标志

Rust 的功能标志提供了一种强大的方式来有条件地编译代码,使您能够为不同的构建配置切换功能。

#### 设置功能标志

在 `Cargo.toml` 中:

```toml
[features]
default = ["perf"]
perf = []
```

#### 在代码中使用

```rust
pub fn process(ctx: Context<'info>) -> ProgramResult {
    #[cfg(not(feature = "perf"))]
    sol_log("Create Class");

    Self::try_from(ctx)?.execute()
}
```

大多数程序会返回指令的名称作为日志,以便更轻松地调试。然而,这种做法成本较高,实际上除了使浏览器更易读和增强调试外并没有必要。

#### 使用不同标志进行构建

```bash
# 启用性能优化(默认)
cargo build-bpf

# 启用额外检查和日志记录
cargo build-bpf --no-default-features
```

---

## 创建 Pinocchio 项目

### 快速开始

创建一个 Pinocchio 

```
vault-program/
├── Cargo.toml
└── src/
    ├── lib.rs           # 入口点和程序主逻辑
    ├── instructions/    # 指令实现
    │   ├── mod.rs
    │   ├── initialize.rs
    │   ├── deposit.rs
    │   └── withdraw.rs
    ├── state/          # 账户状态定义
    │   ├── mod.rs
    │   └── vault.rs
    ├── error.rs        # 错误定义
    └── helpers.rs      # 账户类型辅助工具
```

### 1. Cargo.toml 配置

```toml
[package]
name = "vault-program"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "lib"]

[features]
default = ["perf"]
perf = []

[dependencies]
pinocchio = "0.5"
pinocchio-system = "0.5"
thiserror = { version = "2.0", default-features = false }
num-derive = "0.4"
num-traits = "0.2"

[dev-dependencies]
mollusk-svm = "0.1"
solana-sdk = "2.0"
```

### 2. 错误定义 (src/error.rs)

```rust
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
```

### 3. 账户类型辅助工具 (src/helpers.rs)

```rust
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
use crate::error::VaultError;

/// 账户检查 trait
pub trait AccountCheck {
    fn check(account: &AccountInfo) -> Result<(), ProgramError>;
}

/// 签名者账户
pub struct SignerAccount;

impl AccountCheck for SignerAccount {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if !account.is_signer() {
            return Err(VaultError::NotSigner.into());
        }
        Ok(())
    }
}

/// 系统账户
pub struct SystemAccount;

impl AccountCheck for SystemAccount {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if !account.is_owned_by(&pinocchio_system::ID) {
            return Err(VaultError::InvalidOwner.into());
        }
        Ok(())
    }
}

/// 可写账户检查
pub fn check_writable(account: &AccountInfo) -> Result<(), ProgramError> {
    if !account.is_writable() {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}

/// PDA 验证
pub fn verify_pda(
    account: &AccountInfo,
    seeds: &[&[u8]],
    program_id: &Pubkey,
) -> Result<u8, ProgramError> {
    let (expected_key, bump) = Pubkey::find_program_address(seeds, program_id);
    if account.key() != &expected_key {
        return Err(ProgramError::InvalidSeeds);
    }
    Ok(bump)
}
```

### 4. 金库状态定义 (src/state/vault.rs)

```rust
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
```

### 5. 初始化指令 (src/instructions/initialize.rs)

```rust
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};
use pinocchio_system::{instructions::CreateAccount, system_program};

use crate::{
    error::VaultError,
    helpers::{check_writable, verify_pda, AccountCheck, SignerAccount},
    state::Vault,
};

/// 初始化指令的账户
pub struct InitializeAccounts<'a> {
    /// 金库所有者(签名者,支付者)
    pub owner: &'a AccountInfo,
    /// 金库 PDA 账户
    pub vault: &'a AccountInfo,
    /// 系统程序
    pub system_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for InitializeAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // 解构账户切片
        let [owner, vault, system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // 验证 owner 是签名者
        SignerAccount::check(owner)?;

        // 验证 vault 可写
        check_writable(vault)?;

        // 验证系统程序
        if system_program.key() != &system_program::ID {
            return Err(ProgramError::IncorrectProgramId);
        }

        Ok(Self {
            owner,
            vault,
            system_program,
        })
    }
}

/// 初始化指令
pub struct Initialize<'a> {
    pub accounts: InitializeAccounts<'a>,
}

impl<'a> Initialize<'a> {
    /// 指令判别器
    pub const DISCRIMINATOR: u8 = 0;

    /// 从账户创建指令
    pub fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, ProgramError> {
        let accounts = InitializeAccounts::try_from(accounts)?;
        Ok(Self { accounts })
    }

    /// 执行初始化逻辑
    pub fn process(&self, program_id: &Pubkey) -> ProgramResult {
        // 1. 验证 vault 是正确的 PDA
        let seeds = &[b"vault", self.accounts.owner.key().as_ref()];
        let bump = verify_pda(self.accounts.vault, seeds, program_id)?;

        // 2. 创建 vault 账户
        let rent_lamports = pinocchio_system::rent::Rent::get()?
            .minimum_balance(Vault::LEN);

        CreateAccount {
            from: self.accounts.owner,
            to: self.accounts.vault,
            lamports: rent_lamports,
            space: Vault::LEN as u64,
            owner: program_id,
        }
        .invoke()?;

        // 3. 初始化 vault 数据
        let mut vault_data = self.accounts.vault.try_borrow_mut_data()?;
        let vault = Vault::from_bytes_mut(&mut vault_data)?;
        vault.initialize(self.accounts.owner.key().as_ref(), bump)?;

        #[cfg(not(feature = "perf"))]
        pinocchio::msg!("Vault initialized for owner: {:?}", self.accounts.owner.key());

        Ok(())
    }
}
```

### 6. 存款指令 (src/instructions/deposit.rs)

```rust
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};
use pinocchio_system::instructions::Transfer;

use crate::{
    error::VaultError,
    helpers::{check_writable, verify_pda, AccountCheck, SignerAccount},
    state::Vault,
};

/// 存款指令的账户
pub struct DepositAccounts<'a> {
    /// 存款人(签名者)
    pub owner: &'a AccountInfo,
    /// 金库 PDA 账户
    pub vault: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for DepositAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [owner, vault, _system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // 验证 owner 是签名者
        SignerAccount::check(owner)?;

        // 验证 vault 可写
        check_writable(vault)?;

        Ok(Self { owner, vault })
    }
}

/// 存款指令数据
pub struct DepositInstructionData {
    pub amount: u64,
}

impl TryFrom<&[u8]> for DepositInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // 验证数据长度(8 字节 u64)
        if data.len() != 8 {
            return Err(ProgramError::InvalidInstructionData);
        }

        // 从小端字节转换为 u64
        let amount = u64::from_le_bytes(data.try_into().unwrap());

        // 验证金额大于零
        if amount == 0 {
            return Err(VaultError::InvalidAmount.into());
        }

        Ok(Self { amount })
    }
}

/// 存款指令
pub struct Deposit<'a> {
    pub accounts: DepositAccounts<'a>,
    pub data: DepositInstructionData,
}

impl<'a> Deposit<'a> {
    /// 指令判别器
    pub const DISCRIMINATOR: u8 = 1;

    /// 从账户和数据创建指令
    pub fn try_from(
        data: &[u8],
        accounts: &'a [AccountInfo],
    ) -> Result<Self, ProgramError> {
        let accounts = DepositAccounts::try_from(accounts)?;
        let data = DepositInstructionData::try_from(data)?;
        Ok(Self { accounts, data })
    }

    /// 执行存款逻辑
    pub fn process(&self, program_id: &Pubkey) -> ProgramResult {
        // 1. 验证 vault 是正确的 PDA
        let seeds = &[b"vault", self.accounts.owner.key().as_ref()];
        verify_pda(self.accounts.vault, seeds, program_id)?;

        // 2. 验证 vault 已初始化且所有者正确
        let vault_data = self.accounts.vault.try_borrow_data()?;
        let vault = Vault::from_bytes(&vault_data)?;
        vault.check_initialized()?;
        vault.check_owner(self.accounts.owner.key().as_ref())?;
        drop(vault_data); // 释放借用

        // 3. 执行转账(从 owner 到 vault)
        Transfer {
            from: self.accounts.owner,
            to: self.accounts.vault,
            lamports: self.data.amount,
        }
        .invoke()?;

        #[cfg(not(feature = "perf"))]
        pinocchio::msg!(
            "Deposited {} lamports to vault",
            self.data.amount
        );

        Ok(())
    }
}
```

### 7. 取款指令 (src/instructions/withdraw.rs)

```rust
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};
use pinocchio_system::instructions::{Transfer, transfer};

use crate::{
    error::VaultError,
    helpers::{check_writable, verify_pda, AccountCheck, SignerAccount},
    state::Vault,
};

/// 取款指令的账户
pub struct WithdrawAccounts<'a> {
    /// 取款人(签名者,必须是 vault 所有者)
    pub owner: &'a AccountInfo,
    /// 金库 PDA 账户
    pub vault: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for WithdrawAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [owner, vault, _system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // 验证 owner 是签名者
        SignerAccount::check(owner)?;

        // 验证 vault 可写
        check_writable(vault)?;

        Ok(Self { owner, vault })
    }
}

/// 取款指令数据
pub struct WithdrawInstructionData {
    pub amount: u64,
}

impl TryFrom<&[u8]> for WithdrawInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() != 8 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let amount = u64::from_le_bytes(data.try_into().unwrap());

        if amount == 0 {
            return Err(VaultError::InvalidAmount.into());
        }

        Ok(Self { amount })
    }
}

/// 取款指令
pub struct Withdraw<'a> {
    pub accounts: WithdrawAccounts<'a>,
    pub data: WithdrawInstructionData,
}

impl<'a> Withdraw<'a> {
    /// 指令判别器
    pub const DISCRIMINATOR: u8 = 2;

    /// 从账户和数据创建指令
    pub fn try_from(
        data: &[u8],
        accounts: &'a [AccountInfo],
    ) -> Result<Self, ProgramError> {
        let accounts = WithdrawAccounts::try_from(accounts)?;
        let data = WithdrawInstructionData::try_from(data)?;
        Ok(Self { accounts, data })
    }

    /// 执行取款逻辑
    pub fn process(&self, program_id: &Pubkey) -> ProgramResult {
        // 1. 验证 vault 是正确的 PDA 并获取 bump
        let seeds = &[b"vault", self.accounts.owner.key().as_ref()];
        let bump = verify_pda(self.accounts.vault, seeds, program_id)?;

        // 2. 验证 vault 已初始化且所有者正确
        let vault_data = self.accounts.vault.try_borrow_data()?;
        let vault = Vault::from_bytes(&vault_data)?;
        vault.check_initialized()?;
        vault.check_owner(self.accounts.owner.key().as_ref())?;
        drop(vault_data); // 释放借用

        // 3. 检查余额是否足够
        let vault_balance = self.accounts.vault.lamports();
        let rent_exempt = pinocchio_system::rent::Rent::get()?
            .minimum_balance(Vault::LEN);

        if vault_balance < self.data.amount + rent_exempt {
            return Err(VaultError::InsufficientBalance.into());
        }

        // 4. 执行带签名的转账(从 vault 到 owner)
        // 使用 PDA 签名
        let signer_seeds = &[
            b"vault",
            self.accounts.owner.key().as_ref(),
            &[bump],
        ];

        // 手动构建 CPI 调用
        transfer(
            self.accounts.vault,
            self.accounts.owner,
            self.data.amount,
            &[signer_seeds],
        )?;

        #[cfg(not(feature = "perf"))]
        pinocchio::msg!(
            "Withdrawn {} lamports from vault",
            self.data.amount
        );

        Ok(())
    }
}
```

### 8. 指令模块 (src/instructions/mod.rs)

```rust
pub mod initialize;
pub mod deposit;
pub mod withdraw;

pub use initialize::Initialize;
pub use deposit::Deposit;
pub use withdraw::Withdraw;
```

### 9. 状态模块 (src/state/mod.rs)

```rust
pub mod vault;
pub use vault::Vault;
```

### 10. 主入口点 (src/lib.rs)

```rust
use pinocchio::{
    account_info::AccountInfo,
    entrypoint,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

mod error;
mod helpers;
mod instructions;
mod state;

use instructions::{Deposit, Initialize, Withdraw};

entrypoint!(process_instruction);

/// 程序入口点
/// 
/// 这是所有指令的统一入口,根据第一个字节(discriminator)
/// 来决定调用哪个具体的指令处理器
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // 提取判别器(第一个字节)
    let (discriminator, data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    // 根据判别器路由到对应的指令处理器
    match discriminator {
        // 0: 初始化金库
        &Initialize::DISCRIMINATOR => {
            Initialize::try_from(accounts)?.process(program_id)
        }
        
        // 1: 存款
        &Deposit::DISCRIMINATOR => {
            Deposit::try_from(data, accounts)?.process(program_id)
        }
        
        // 2: 取款
        &Withdraw::DISCRIMINATOR => {
            Withdraw::try_from(data, accounts)?.process(program_id)
        }
        
        // 未知指令
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

#[cfg(test)]
mod tests;
```

### 11. 测试 (src/tests.rs)

```rust
use pinocchio::{account_info::AccountInfo, pubkey::Pubkey};
use mollusk_svm::Mollusk;
use solana_sdk::{
    account::{Account, AccountSharedData},
    instruction::{AccountMeta, Instruction},
};

use crate::state::Vault;

#[test]
fn test_initialize_vault() {
    // 创建 Mollusk 测试环境
    let program_id = Pubkey::new_unique();
    let mollusk = Mollusk::new(&program_id, "target/deploy/vault_program");

    // 创建测试账户
    let owner = Pubkey::new_unique();
    let (vault_pda, _bump) = Pubkey::find_program_address(
        &[b"vault", owner.as_ref()],
        &program_id,
    );

    // 构建初始化指令
    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(owner, true),           // owner (signer)
            AccountMeta::new(vault_pda, false),      // vault PDA
            AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
        ],
        data: vec![0], // discriminator = 0 (Initialize)
    };

    // 执行测试
    // (这里需要根据 Mollusk 的实际 API 来完成)
    // let result = mollusk.process_instruction(&instruction, &accounts);
    // assert!(result.is_ok());
}

#[test]
fn test_deposit() {
    // 类似的测试逻辑...
}

#[test]
fn test_withdraw() {
    // 类似的测试逻辑...
}
```

### 概念串联总结

这个完整示例展示了 Pinocchio 程序开发的所有核心概念:

1. **入口点** (`lib.rs`): 使用 `entrypoint!` 宏和 discriminator 路由
2. **TryFrom 验证** (所有 instructions): 账户和数据的类型安全验证
3. **账户类型** (`helpers.rs`): 使用 trait 实现可重用的验证逻辑
4. **零拷贝** (`vault.rs`): 直接从字节切片读取数据,无需反序列化
5. **CPI 调用** (`deposit.rs`, `withdraw.rs`): 简洁的跨程序调用
6. **PDA 签名** (`withdraw.rs`): 使用 `invoke_signed` 进行授权转账
7. **错误处理** (`error.rs`): 使用 `thiserror` 定义清晰的错误类型
8. **性能优化**: 使用 `perf` feature flag 控制日志输出

### 使用方式

```bash
# 构建程序
cargo build-bpf

# 运行测试
cargo test-sbf

# 构建生产版本(启用性能优化)
cargo build-bpf --release

# 构建调试版本(包含日志)
cargo build-bpf --no-default-features
```

---

## 测试

在主网部署之前,进行彻底的测试是至关重要的,以识别潜在的漏洞和问题。

经过充分测试的程序可以:
- 防止财务损失
- 建立用户信任
- 确保应用程序在各种条件下正常运行

### Mollusk 测试

当设置复杂的程序状态或需要复杂的链上交互变得困难时,Mollusk 提供了对测试环境更细致的控制。

**Mollusk** 是一个专为 Solana 程序设计的 Rust 测试框架,它可以让你:

- 在没有网络开销的情况下独立测试程序逻辑
- 轻松设置复杂的账户状态和程序条件
- 比完整的集成测试运行速度更快
- 模拟特定的区块链条件和边界情况

### 设置测试

在 `lib.rs` 中使用 test 配置标志导入测试模块:

```rust
#[cfg(test)]
pub mod tests;
```

### 运行测试

```bash
cargo test-sbf
```

---

## 总结

恭喜你!你已经完成了 Pinocchio 入门课程。现在,你已经对 Pinocchio 的工作原理有了扎实的基础,从核心概念到实际实现细节都有了深入了解。

### 你学到了什么

在本课程中,你掌握了以下重要知识:

1. **Pinocchio 的基础知识** - 理解零拷贝操作和极简设计
2. **如何构建 Pinocchio 程序** - 从入口点到指令处理
3. **理解 discriminators、TryFrom traits、账户和指令** - 手动验证的最佳实践
4. **构建安全的 Solana 程序的最佳实践** - 错误处理、性能优化和测试

### 下一步

现在,你已经准备好开始构建你的第一个 Pinocchio 程序了!巩固知识的最佳方式是通过实践操作。我们鼓励你:

1. 从适合初学者的练习开始
2. 构建并测试你的第一个 Pinocchio 程序
3. 加入社区,分享你的进展并获得帮助

记住,每一位优秀的开发者都是从他们的第一个程序开始的。不要害怕尝试和犯错,这正是我们学习和成长的方式!

---

## 参考资源

- [Pinocchio GitHub 仓库](https://github.com/febo/pinocchio)
- [Solana 官方文档](https://docs.solana.com/)
- [Anchor 框架文档](https://www.anchor-lang.com/)

---

*本文档最后更新于 2026年1月30日*
