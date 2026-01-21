---
title: 第五节-使用 Anchor 重写简单链上数据存储程序的代码
tags: [Solana]
---

## Anchor 环境搭建

在本书的中篇内容里，我们将第一个想法带上 Solana，我们编写了一个可以存储任意数据的简单数据存储程序。我们使用原生 Rust 编写了这个程序，但过程中需要我们直面账户校验、序列化和客户端打包这些杂事，往往会消磨兴致。Anchor 出场的意义，正是把这些粗重活接过去，让你把精力放在要实现什么，而不是怎么让代码"跑"起来。

Anchor 是一种为 Solana 区块链设计的开发框架，用于快速、安全地构建和部署链上程序。它通过提供工具和抽象来简化开发流程，包括自动处理账户和指令数据的序列化、内置安全检查、生成客户端库以及提供测试工具。

我们在这里用 Anchor 重写那个数据存储程序，让你体会它的魔力。我们不会在这里做说明书式的工具介绍，如果您需要它，请参考官方文档。我们只会准备一张干净的工作台来组装代码，让你专注于实现核心功能。你会看到 Anchor 的核心心智模型，完成一次从零到一的本地运行，并学会辨认路上的几个小坑。

### 历史

Anchor 最初由 Project Serum（由 FTX 交易所主导）团队开发，旨在简化 Solana 上的智能合约开发。在 Solana 生态早期，大家通常使用 solana-program 直接编写原生 Rust 程序，但面临一些问题：

- **大量的样板代码**：开发者需要编写大量重复的代码来处理账户验证、PDA 账户管理、租赁豁免管理等琐事。
- **安全性挑战**：直接操作低级账户和指令数据容易引入安全漏洞，需要开发者具备深厚的 Solana 内部知识。

Anchor 通过引入高级抽象和自动化工具，大大简化了这些任务。它大量使用宏和属性来自动生成样板代码以防止常见漏洞，并生成易于使用的客户端库。

不过后续随着 2022 年 11 月 FTX 交易所崩盘，Project Serum 团队解体，Anchor 的维护也一度陷入停滞。原 Serum 团队部分成员成立了 coral-xyz，Anchor 的仓库迁移到了 https://github.com/coral-xyz/anchor。在 2025 年 4 月，Solana 开发团队经历了一次重大人事变动，Solana 协议的核心客户端 solana 改名 agave 并由 solana-labs 转移给了 anza-xyz 团队；Anchor 则是由 coral-xyz 转移给了 solana-foundation 维护：https://github.com/solana-foundation/anchor。

> 2025 年 4 月这次人事变动看起来范围相当巨大。

### 环境搭建

如果你的机器还没有这些工具，请先安好：Rust、Solana CLI、Node.js 与 Yarn，以及 Anchor 本体。下面的命令可以直接复用；若你已有其一，可跳过相应小节。

**安装 Anchor（使用 avm 管理版本）：**

```bash
$ cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
$ avm install latest
$ avm use latest
$ anchor --version
```

**准备 Solana CLI 与本地链：**

```bash
$ sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
$ solana --version
$ solana config set --url http://127.0.0.1:8899
$ solana-test-validator -r
```

**准备 Node.js 与 Yarn**，因为 Anchor 的测试与客户端默认使用 TypeScript：

```bash
$ npm install -g yarn
```

本章节的配套代码在这里。如果你正在阅读该配套仓库，`Anchor.toml` 已预设本地网络与钱包路径，`tests/` 里也放好了 TypeScript 的测试脚本。进入仓库根目录，装上依赖即可：

```bash
$ yarn install
```

**小提示**：第一次跑本地链时，别忘了给默认钱包要点启动资金。

```bash
$ solana airdrop 2
```

### 创建项目

我们先使用 Anchor 搭一个最小可用的程序，看看它长什么样。

```bash
$ anchor init pxsol-ss-anchor
$ cd pxsol-ss-anchor
```

脚手架会生成一套目录：

- `programs/<name>/src/lib.rs` 是合约入口。你会看到 `#[program]` 模块和一两个演示方法。
- `Anchor.toml` 是配置中心，记录 program id、要连接的集群、测试脚本等。
- `tests/` 放着 TypeScript 测试，等会儿它会代表客户端来与节点进行交互和测试。

先试着构建它：

```bash
$ anchor build
```

如果你还没启动本地链，开一个终端让验证器常驻：

```bash
$   
```

接着跑一次测试：

```bash
$ anchor test --skip-local-validator
```

补充说明：`anchor test --skip-local-validator` 内部大致会按下面顺序做事（不同 Anchor 版本在细节上可能略有差异，但整体流程一致）：

1. **Build**：先构建程序（等价于先跑一次 `anchor build`），生成 `target/deploy/*.so` 和对应的 IDL 等产物。
2. **Deploy 到 cluster**：把刚构建出来的程序部署到 `Anchor.toml` 里 `[provider].cluster` 指定的集群（本项目是 `localnet`）。
3. **Run tests**：执行 `Anchor.toml` 的 `[scripts].test` 命令来跑 TypeScript 测试。本项目配置的是：
   - `yarn run ts-mocha -p ./tsconfig.json -t 1000000 "tests/**/*.ts"`

其中 `--skip-local-validator` 的意思是：**不要让 `anchor test` 自动启动本地 validator**。因此你需要提前手动启动 `solana-test-validator -r`（或者你去掉该参数，让 `anchor test` 自己起本地链）。

这条命令做了三件事：

1. 构建 Rust 程序
2. 把它部署到本地链
3. 运行 `tests/` 下的 TypeScript 测试用例

### 如何开始

当我们开始实现真正的业务，可以沿着这条最小路径前进：

1. 在 `programs/<name>/src/lib.rs`：
   - 定义 PDA 账户的数据结构，使用 `#[account]` 标记它们。
   - 新增一些方法，编写业务逻辑，使用 `Context<>` 定义每个方法需要的账户。
   - 写出期望的 accounts 结构与约束。
2. 在 `tests/` 写一个最小的调用脚本，跑 `anchor test` 观察失败信息。
3. 循环填写逻辑代码，补齐账户、空间与权限，并时刻调整测试脚本，直到测试通过。
4. 最后接入前端或后端服务。

当你跨过这些门槛，Anchor 就会像一把顺手的扳手。你不用每天都去记 Torx 和内六角的尺寸，只管拧紧你真正关心的那颗螺丝。


---

## Anchor 里的简单数据存储合约

本章节的配套代码在这里。

这一节，我们用 Anchor 实现一个数据存储合约，走一遍从建模到程序构建的过程。你会看到三个关键点：账户心智模型、两条指令（init/update）、以及动态重分配与租金的细枝末节。代码出自 `programs/pxsol-ss-anchor/src/lib.rs`，但我们以文字的方式来理解它。

### 数据存储格式设计

我们知道用户数据实际上是存储在 PDA 程序扩展账户里的。在我们使用原生 Rust 编写该程序时，我们其实并没有对 PDA 账户里的数据格式做过多约束，只要能序列化与反序列化就行。但在 Anchor 里，我们可以定义一个结构体来描述它，并用 `#[account]` 标记它。这种做法可以方便我们后续的开发，也方便我们对链上数据的分析和理解。

```rust
#[account]
pub struct Data {
    pub auth: Pubkey,  // The owner of this pda account
    pub bump: u8,      // The bump to generate the PDA
    pub data: Vec<u8>  // The content, arbitrary bytes
}

impl Data {
    pub fn space_for(data_len: usize) -> usize {
        // 8 (discriminator) + 32 (auth) + 1 (bump) + 4 (vec len) + data_len
        8 + 32 + 1 + 4 + data_len
    }
}
```

方法 `space_for()` 用来计算账户所需空间。这里的空间由五部分组成。我们需要使用该函数来计算账户的租赁豁免金额。

Anchor 生成的 PDA 账户，账户数据的前 8 字节固定用来标识账户的具体类型，保证 Anchor 在反序列化时类型安全。它的算法是对字符串 `"account:Data"` 做 SHA256 哈希，取前 8 字节。这 8 个字节被称作 **account discriminator**。您可以用下面的 Python 代码来计算它：

```python
import hashlib

r = hashlib.sha256(b'account:Data').digest()[:8]
print(list(r))  # [206, 156, 59, 188, 18, 79, 240, 232]
```

举一个实际的例子，假设我们要存储 "Hello World!" 字节的数据，那么我们实际存储在 PDA 账户里的内容是：

```
discriminator: [206, 156, 59, 188, 18, 79, 240, 232]
auth: [32 bytes of auth pubkey]
bump: [1 byte of bump]
data_len: [12, 0, 0, 0]
data: [72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 33]
```

### 指令：初始化程序扩展账户

我们设计了两条指令：`init` 和 `update`。`init` 用来初始化程序扩展账户，`update` 用来更新内容。下面我们逐一分析它们的实现。

指令 `init` 做了三件事：记住谁是拥有者、记录 bump、并把内容置空。

```rust
pub fn init(ctx: Context<Init>) -> Result<()> {
    let account_user = &ctx.accounts.user;
    let account_user_pda = &mut ctx.accounts.user_pda;
    account_user_pda.auth = account_user.key();
    account_user_pda.bump = ctx.bumps.user_pda;
    account_user_pda.data = Vec::new();
    Ok(())
}
```

**关于 `Init` 的要求：**

这里的 `Init` 不能是随意定义的结构体，它必须满足以下要求：

1. **必须用 `#[derive(Accounts)]` 宏标记**：这个宏会为结构体生成必要的 trait 实现，让 Anchor 能够解析账户、校验约束、反序列化数据等。
2. **必须有生命周期参数 `'info`**：表示账户引用在整个指令执行期间都有效。
3. **字段必须是 Anchor 支持的账户类型**：如 `Account<'info, T>`、`Signer<'info>`、`Program<'info, T>`、`AccountInfo<'info>` 等，不能使用普通 Rust 类型（如 `String`、`u32` 等）。

如果不满足这些要求，编译器会报错。详细的 `Init` 定义见下文。

**关于 `Context<Init>` 参数：**

`Context` 是 Anchor 框架提供的泛型结构体，它封装了指令执行时需要的所有上下文信息：

- **`ctx.accounts`**：类型是 `Init`（你自己定义的账户列表结构体），包含了这条指令需要的所有账户（如 `user`、`user_pda`、`system_program`）。Anchor 在调用你的函数之前，会先解析交易传入的账户，校验所有 `#[account(...)]` 约束，然后把结果封装到这里。
- **`ctx.bumps`**：一个 `BTreeMap<String, u8>`，存储 Anchor 自动计算出的 PDA bump 值。当你在账户约束里写 `bump`（不指定具体值）时，Anchor 会自动求解 bump 并存到这里。你可以通过 `ctx.bumps.user_pda` 访问名为 `user_pda` 的账户对应的 bump。
- **`ctx.program_id`**：当前程序的 ID（pubkey）。
- **`ctx.remaining_accounts`**：额外传入但未在账户结构体中声明的账户列表。

泛型参数 `Init` 是你自己定义的账户列表类型（见下文），`Context<Init>` 把 Anchor 的上下文容器和你的账户定义组合起来，让你能通过 `ctx.accounts.user` 这样的方式直接访问类型安全的账户。

`Context` 在 Anchor 框架中的定义（简化版）：

```rust
pub struct Context<'a, 'b, 'c, 'info, T> {
    /// 当前程序的 ID
    pub program_id: &'a Pubkey,
    
    /// 账户列表，类型是泛型参数 T（你定义的账户结构体，如 Init）
    pub accounts: &'b mut T,
    
    /// 额外传入但未在账户结构体中声明的账户
    pub remaining_accounts: &'c [AccountInfo<'info>],
    
    /// Anchor 自动计算的 PDA bump 值映射表
    pub bumps: BTreeMap<String, u8>,
}
```

其中生命周期参数 `'info` 表示账户引用在整个指令执行期间都有效，泛型参数 `T` 就是你自己定义的账户列表结构（如 `Init`）。

在设计好指令后，我们需要定义它的账户列表以及账户约束。

```rust
#[derive(Accounts)]
pub struct Init<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        seeds = [SEED, user.key().as_ref()],
        bump,
        space = Data::space_for(0)
    )]
    pub user_pda: Account<'info, Data>,
    pub system_program: Program<'info, System>,
}
```

此时程序扩展账户里的数据字段是空的，但已具备完整身份与归属，并达成了租赁豁免。

下面介绍下这些账户的含义以及通过 `#[account(...)]` 规定的约束：

**user** 是调用者。
- `Signer<'info>` 表示账户的基本类型，即需要签名，因为它要支付创建账户的租金与手续费。
- `#[account(mut)]` 表示可写。

**user_pda** 是要新建的 PDA 账户
- `Account<Data>` 表示账户的基本类型
- `#[account(init)]` 标记表示该账户需要在本次指令中被创建。这里的"本次指令"指的是**客户端调用使用了这个结构体（`Init`）的指令函数（`init`）**时。注意：函数名可以不叫 `init`，关键是使用了带有 `init` 约束的结构体。Anchor 会在进入函数体之前，先自动创建这个账户（调用系统程序的 create_account）。
- `#[account(payer = user)]` 标记创建 user_pda 时的租金与手续费由 user 支付。
- `#[account(seeds = [SEED, user.key().as_ref()])]` PDA 的种子数组，这里用常量 seed 与 user 公钥派生唯一地址。
- `#[account(bump)]` 让 Anchor 自动求解并记录该 PDA 的 bump，用于签名和地址唯一性。通常总是和 seeds 一起使用。
- `#[account(space = Data::space_for(0))]` 为账户预留的字节数。

**system_program** 是系统合约。
- `Program<'info, System>` 表示系统程序的账户，供 Anchor 代为调用系统指令（如创建账户、转账、分配空间）。

### 指令：存储或更新数据

更新内容时，我们允许程序扩展账户变大或变小。变大需要补齐租金，变小则把多出来的 lamports 退给拥有者。逻辑可以读作三步：验权、重分配、找零。Anchor 框架会帮我们处理租赁豁免与扣费，但找零需要我们自己来做。也就是当新数据比老数据大时，我们不需要做什么，Anchor 会自动帮我们补齐租赁豁免资金；但当新数据比老数据小时，我们要把多出来的 lamports 退给拥有者。

```rust
pub fn update(ctx: Context<Update>, data: Vec<u8>) -> Result<()> {
    let account_user = &ctx.accounts.user;
    let account_user_pda = &mut ctx.accounts.user_pda;

    // Update the data field with the new data.
    account_user_pda.data = data;

    // If the account was shrunk, Anchor won't automatically refund excess lamports. Refund any surplus (over the
    // new rent-exempt minimum) back to the user.
    let account_user_pda_info = account_user_pda.to_account_info();
    let rent_exemption = Rent::get()?.minimum_balance(account_user_pda_info.data_len());
    let hold = **account_user_pda_info.lamports.borrow();
    if hold > rent_exemption {
        let refund = hold.saturating_sub(rent_exemption);
        **account_user_pda_info.lamports.borrow_mut() = rent_exemption;
        **account_user.lamports.borrow_mut() = account_user.lamports().checked_add(refund).unwrap();
    }
    Ok(())
}
```

相配套的账户约束清晰地约定了该指令的一些策略细节。

```rust
#[derive(Accounts)]
#[instruction(new_data: Vec<u8>)]
pub struct Update<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [SEED, user.key().as_ref()],
        bump = user_pda.bump,
        realloc = Data::space_for(new_data.len()),
        realloc::payer = user,
        realloc::zero = false,
        constraint = user_pda.auth == user.key() @ PxsolError::Unauthorized,
    )]
    pub user_pda: Account<'info, Data>,
    pub system_program: Program<'info, System>,
}
```

下面介绍这些账户的含义以及通过 `#[account(...)]` 规定的约束：

**user** 是调用者。
- `Signer<'info>` 表示账户的基本类型，即需要签名。
- `#[account(mut)]` 表示可写，因为如果 PDA 账户扩容，user 需要补缴租金；如果 PDA 账户缩小，多余的 lamports 会退还给 user。

**user_pda** 是要更新的 PDA 账户
- `Account<Data>` 表示账户的基本类型
- `#[account(mut)]` 表示可写，因为我们要修改其中的数据内容。
- `#[account(seeds = [SEED, user.key().as_ref()])]` PDA 的种子数组，必须与创建时一致，用于验证地址派生的正确性。
- `#[account(bump = user_pda.bump)]` 使用之前存储在账户中的 bump 值，确保 PDA 地址的唯一性与合法性。这个 bump 是在 init 时记录的。
- `#[account(realloc = Data::space_for(new_data.len()))]` 动态重新分配账户空间。Anchor 会根据新数据的长度自动调整账户大小。如果新空间大于旧空间，会从 realloc::payer 扣除额外的租金；如果新空间小于旧空间，账户会缩小，但多余的 lamports 不会自动退还（需要在指令逻辑中手动处理）。
- `#[account(realloc::payer = user)]` 指定当账户需要扩容时，由 user 支付额外的租金。如果 user 余额不足，交易会失败。
- `#[account(realloc::zero = false)]` 表示重新分配空间时不需要将新增的字节清零。设为 false 可以节省计算单元，因为我们会立即用新数据覆盖这些字节。如果您需要确保新增空间初始化为零，应设为 true。
- `#[account(constraint = user_pda.auth == user.key() @ PxsolError::Unauthorized)]` 自定义约束检查，验证调用者 user 的公钥必须与 PDA 账户中存储的 auth 字段一致。如果不一致，会抛出 PxsolError::Unauthorized 错误。这是一个关键的权限检查，确保只有账户的拥有者才能更新数据。

**system_program** 是系统合约。
- `Program<'info, System>` 表示系统程序的账户，用于账户重新分配和 lamports 转账操作。

### 收束

我们 Anchor 版本的数据存储器很平凡，却把 Anchor 最常用的几块能力都连接在了一起：账户约束、动态重分配、PDA 代签。把它跑通之后，我们可以继续加上一些更加复杂的逻辑。代码总共只有不到 100 行，但它是个很好的起点。您应该能很快阅读懂它，因此在这里不再过多赘述。


---

## Anchor 测试框架

当你写下第一行合约代码，测试就是它开口说的第一句话。我们希望它既能在框架下顺畅表达，也能在底层协议里自证严谨。本节把测试当成一段小旅程：先用 Anchor 自带的 TypeScript 测试框架走一条铺好的大道，再用 Python 下的 pxsol 客户端走一条原野小路（直接按二进制协议构造交易数据）。

目标很朴素：在本地链上，初始化一个数据存储器，多次更新内容，然后把它读回来确认数据无误。路径与代码都在仓库的 `tests/` 目录里。

### TypeScript

这条路最省心。你只需要告诉 Anchor：我要哪个程序，要调用这个程序的哪个指令，带上哪些账户与参数。其余的编解码与账户核验，由 Anchor 和 IDL 替你完成。

Anchor 的 IDL 会在你第一次构建程序时自动生成。它记录了程序 ID、每个指令的账户与参数、以及每个账户的数据结构。你可以把它想象成一个桥梁，连接链上程序与链下客户端。

我们的测试很比较简单，先调用一次 `init`，然后调用两次 `update`，每次都传入不同长度的内容。每次调用后，我们都 fetch 一次账户数据，确认内容正确。

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PxsolSsAnchor } from "../target/types/pxsol_ss_anchor";

describe("pxsol-ss-anchor", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.pxsolSsAnchor as Program<PxsolSsAnchor>;
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const wallet = provider.wallet as anchor.Wallet;
  const walletPda = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("data"), wallet.publicKey.toBuffer()],
    program.programId
  )[0];

  it("Init with content and then update (grow and shrink)", async () => {
    // Airdrop SOL to fresh authority to fund rent and tx fees
    await provider.connection.confirmTransaction(await provider.connection.requestAirdrop(
      wallet.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    ), "confirmed");

    const poemInitial = Buffer.from("");
    const poemEnglish = Buffer.from("The quick brown fox jumps over the lazy dog");
    const poemChinese = Buffer.from("片云天共远, 永夜月同孤.");
    const walletPdaData = async (): Promise<Buffer<ArrayBuffer>> => {
      let walletPdaData = await program.account.data.fetch(walletPda);
      return Buffer.from(walletPdaData.data);
    }

    await program.methods
      .init()
      .accounts({
        user: wallet.publicKey,
        userPda: walletPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([wallet.payer])
      .rpc();
    if (!(await walletPdaData()).equals(poemInitial)) throw new Error("mismatch");

    await program.methods
      .update(poemEnglish)
      .accounts({
        user: wallet.publicKey,
        userPda: walletPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([wallet.payer])
      .rpc();
    if (!(await walletPdaData()).equals(poemEnglish)) throw new Error("mismatch");

    await program.methods
      .update(poemChinese)
      .accounts({
        user: wallet.publicKey,
        userPda: walletPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([wallet.payer])
      .rpc();
    if (!(await walletPdaData()).equals(poemChinese)) throw new Error("mismatch");
  });
});
```

运行：

```bash
# 自动构建、部署到本地链并运行 TypeScript 测试
$ anchor test
```

### Python Pxsol

这条路更贴近协议本身。我们会亲手排列账户列表，拼接 8 字节方法 discriminator，再把 4 字节小端长度与原始字节流接在后头。它适合跨语言集成，或在没有 Anchor 客户端的环境里验算每一步。

与 account discriminator 不同，方法 discriminator 是对方法名称做 SHA256 哈希后取前 8 字节。例如 `init` 方法的 discriminator 是 `dc3bcfec6cfa2f64`（十六进制）。`update` 方法的 discriminator 是 `dbc858b09e3ffd7f`（十六进制）。

```python
import hashlib

r = hashlib.sha256(b'global:init').digest()[:8]
print(list(r))  # [220, 59, 207, 236, 108, 250, 47, 100]
r = hashlib.sha256(b'global:update').digest()[:8]
print(list(r))  # [219, 200, 88, 176, 158, 63, 253, 127]
```

代码如下：

```python
import argparse
import base64
import pxsol


parser = argparse.ArgumentParser()
parser.add_argument('--net', type=str, choices=['develop', 'mainnet', 'testnet'], default='develop')
parser.add_argument('--prikey', type=str, default='11111111111111111111111111111112')
parser.add_argument('args', nargs='+')
args = parser.parse_args()

user = pxsol.wallet.Wallet(pxsol.core.PriKey.base58_decode(args.prikey))
prog_pubkey = pxsol.core.PubKey.base58_decode('GS5XPyzsXRec4sQzxJSpeDYHaTnZyYt5BtpeNXYuH1SM')
data_pubkey = prog_pubkey.derive_pda(b'data' + user.pubkey.p)[0]


def init():
    rq = pxsol.core.Requisition(prog_pubkey, [], bytearray())
    rq.account.append(pxsol.core.AccountMeta(user.pubkey, 3))
    rq.account.append(pxsol.core.AccountMeta(data_pubkey, 1))
    rq.account.append(pxsol.core.AccountMeta(pxsol.program.System.pubkey, 0))
    rq.data = bytearray().join([
        bytearray([220, 59, 207, 236, 108, 250, 47, 100]),
    ])
    tx = pxsol.core.Transaction.requisition_decode(user.pubkey, [rq])
    tx.message.recent_blockhash = pxsol.base58.decode(pxsol.rpc.get_latest_blockhash({})['blockhash'])
    tx.sign([user.prikey])
    txid = pxsol.rpc.send_transaction(base64.b64encode(tx.serialize()).decode(), {})
    pxsol.rpc.wait([txid])
    r = pxsol.rpc.get_transaction(txid, {})
    for e in r['meta']['logMessages']:
        print(e)


def update():
    rq = pxsol.core.Requisition(prog_pubkey, [], bytearray())
    rq.account.append(pxsol.core.AccountMeta(user.pubkey, 3))
    rq.account.append(pxsol.core.AccountMeta(data_pubkey, 1))
    rq.account.append(pxsol.core.AccountMeta(pxsol.program.System.pubkey, 0))
    rq.data = bytearray().join([
        bytearray([219, 200, 88, 176, 158, 63, 253, 127]),
        len(args.args[1].encode()).to_bytes(4, 'little'),
        args.args[1].encode(),
    ])
    tx = pxsol.core.Transaction.requisition_decode(user.pubkey, [rq])
    tx.message.recent_blockhash = pxsol.base58.decode(pxsol.rpc.get_latest_blockhash({})['blockhash'])
    tx.sign([user.prikey])
    txid = pxsol.rpc.send_transaction(base64.b64encode(tx.serialize()).decode(), {})
    pxsol.rpc.wait([txid])
    r = pxsol.rpc.get_transaction(txid, {})
    for e in r['meta']['logMessages']:
        print(e)


def load():
    info = pxsol.rpc.get_account_info(data_pubkey.base58(), {})
    print(base64.b64decode(info['data'][0])[8 + 32 + 1 + 4:].decode())


if __name__ == '__main__':
    eval(f'{args.args[0]}()')
```

运行：

```bash
$ solana-test-validator -l /tmp/solana-ledger
$ anchor deploy
# Program Id: GS5XPyzsXRec4sQzxJSpeDYHaTnZyYt5BtpeNXYuH1SM

$ python tests/pxsol-ss-anchor.py init
$ python tests/pxsol-ss-anchor.py update "The quick brown fox jumps over the lazy dog"
$ python tests/pxsol-ss-anchor.py load
# The quick brown fox jumps over the lazy dog

$ python tests/pxsol-ss-anchor.py update "片云天共远, 永夜月同孤."
$ python tests/pxsol-ss-anchor.py load
# 片云天共远, 永夜月同孤.
```
