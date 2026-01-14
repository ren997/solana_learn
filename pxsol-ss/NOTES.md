---
title: 第三节-Solana程序开发入门-简单链上数据存储程序
tags: Solana 教程
---

这一节主要讲解简单的链上数据存储程序开发。

## 参考资料

- [OpenBuild 课程资料](https://openbuild.xyz/learn/challenges/2086624241/1766375449)
- [Solana 官方安装文档](https://solana.com/zh/docs/intro/installation)
- [链上存储程序课程资料](https://accu.cc/content/solana/ss_rust_env/)
- [完整源码 GitHub](https://github.com/mohanson/pxsol-ss)

## 环境安装

### 1. 安装 WSL

按照官方文档安装 WSL 即可。

### 2. 安装 Solana 开发环境

这个命令会一键安装 Rust、Solana CLI、Anchor 等所有依赖：

```bash
curl --proto '=https' --tlsv1.2 -sSfL https://solana-install.solana.workers.dev | bash
```

### 3. WSL2 代理配置（重要）

如果你使用 WSL2，需要单独配置代理。因为 WSL2 是独立的虚拟机，Windows 的全局代理对它不生效。

**前提条件：**
- 代理软件（如 Clash）需要开启「允许局域网连接」

**配置方法：**

执行以下命令，将代理配置写入 `~/.bashrc`：

```bash
cat >> ~/.bashrc << 'EOF'

# WSL2 代理设置
export hostip=$(cat /etc/resolv.conf | grep nameserver | awk '{print $2}')
export http_proxy="http://${hostip}:7897"
export https_proxy="http://${hostip}:7897"
export all_proxy="http://${hostip}:7897"
EOF
```

然后使配置生效：

```bash
source ~/.bashrc
```

> 注意：`7897` 是 Clash 的默认端口，如果你的代理端口不同，请自行修改。

**验证代理是否生效：**

```bash
curl -I https://google.com
```

![安装完成截图]({{ site.baseurl }}/assets/images/sol3/img.png)

验证版本时，Anchor 可能会报错：

```
Error: Anchor version not set. Please run `avm use latest`.
```

执行以下命令设置 Anchor 版本即可：

```bash
avm use latest
```

![验证成功截图]({{ site.baseurl }}/assets/images/sol3/img_1.png)

---

## 为什么用 Rust 写 Solana 合约

Solana 的运行时基于 BPF（Berkeley Packet Filter），而不是 EVM。BPF 是一种高效的沙盒执行环境。

Rust 是编写 Solana 合约的最佳选择：
- 编译器对内存和类型检查严格，减少低级错误
- 性能接近 C/C++，但更安全
- Solana 官方 SDK 用 Rust 编写，生态完善

## 项目目标：链上数据存储器

我们要实现一个支持任意用户创建、更新、扩容和缩容数据账户的 Solana 程序：

1. **初始化数据账户**：程序为用户创建 PDA 作为数据存储账户，根据数据长度自动分配存储空间
2. **更新数据内容**：数据变长时补足租金，数据变短时退还多余租金

## 搭建项目结构

### 创建项目

```bash
cargo new --lib pxsol-ss
cd pxsol-ss
```

### 配置 Cargo.toml

```toml
[package]
name = "pxsol-ss"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "lib"]

[dependencies]
solana-program = "2"
```

- `cdylib`：编译为 .so 文件，用于部署到链上
- `lib`：编译为普通 Rust 库，用于本地测试

### 目录结构

```
pxsol-ss/
├── Cargo.toml
└── src/
    └── lib.rs
```

## 入口函数解析

Solana 交易中的每个指令包含三部分：

```rust
pub struct Instruction {
    pub program_id: Pubkey,        // 程序地址
    pub accounts: Vec<AccountMeta>, // 账户列表
    pub data: Vec<u8>,             // 指令数据
}
```

程序入口函数：

```rust
solana_program::entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &solana_program::pubkey::Pubkey,  // 当前程序地址
    accounts: &[solana_program::account_info::AccountInfo],  // 账户列表
    data: &[u8],  // 指令数据
) -> solana_program::entrypoint::ProgramResult {
    // 处理逻辑
    Ok(())
}
```

**参数说明：**
- `program_id`：当前合约的地址，可用于校验 PDA
- `accounts`：调用方传入的账户列表，程序只能使用这些账户
- `data`：自定义指令数据，类似函数调用参数

## 创建数据账户

### 涉及的账户

| 索引 | 角色 | 签名 | 可写 |
|------|------|------|------|
| 0 | 用户钱包账户 | ✅ | ✅ |
| 1 | 用户数据账户(PDA) | ❌ | ✅ |
| 2 | System 程序 | ❌ | ❌ |
| 3 | Sysvar Rent | ❌ | ❌ |

### 获取账户

```rust
let accounts_iter = &mut accounts.iter();
let account_user = solana_program::account_info::next_account_info(accounts_iter)?;
let account_data = solana_program::account_info::next_account_info(accounts_iter)?;
let _ = solana_program::account_info::next_account_info(accounts_iter)?; // System
let _ = solana_program::account_info::next_account_info(accounts_iter)?; // Sysvar rent
```

### 计算租赁豁免

```rust
let rent_exemption = solana_program::rent::Rent::get()?.minimum_balance(data.len());
```

### 派生 PDA 地址

```rust
let calculated_pda = solana_program::pubkey::Pubkey::find_program_address(
    &[&account_user.key.to_bytes()], 
    program_id
);
assert_eq!(account_data.key, &calculated_pda.0);
let bump_seed = calculated_pda.1;
```

### 创建 PDA 账户

```rust
solana_program::program::invoke_signed(
    &solana_program::system_instruction::create_account(
        account_user.key,
        account_data.key,
        rent_exemption,
        data.len() as u64,
        program_id,
    ),
    accounts,
    &[&[&account_user.key.to_bytes(), &[bump_seed]]],
)?;
```

> `invoke_signed` vs `invoke`：PDA 没有私钥，需要用 `invoke_signed` 让程序代表 PDA 签名。

### 写入数据

```rust
account_data.data.borrow_mut().copy_from_slice(data);
```

## 动态租赁调节

### 更新数据

```rust
// 重新分配空间
account_data.realloc(data.len(), false)?;
// 写入新数据
account_data.data.borrow_mut().copy_from_slice(data);
```

### 租金补足（数据变长）

```rust
if rent_exemption > account_data.lamports() {
    solana_program::program::invoke(
        &solana_program::system_instruction::transfer(
            account_user.key,
            account_data.key,
            rent_exemption - account_data.lamports(),
        ),
        accounts,
    )?;
}
```

### 租金退款（数据变短）

```rust
if rent_exemption < account_data.lamports() {
    **account_user.lamports.borrow_mut() = account_user.lamports() + account_data.lamports() - rent_exemption;
    **account_data.lamports.borrow_mut() = rent_exemption;
}
```

> 退款不需要 `transfer` 指令，因为程序是 PDA 的 owner，可以直接修改余额。

## 完整链上代码

```rust
#![allow(unexpected_cfgs)]

use solana_program::sysvar::Sysvar;

solana_program::entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &solana_program::pubkey::Pubkey,
    accounts: &[solana_program::account_info::AccountInfo],
    data: &[u8],
) -> solana_program::entrypoint::ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let account_user = solana_program::account_info::next_account_info(accounts_iter)?;
    let account_data = solana_program::account_info::next_account_info(accounts_iter)?;
    let _ = solana_program::account_info::next_account_info(accounts_iter)?;
    let _ = solana_program::account_info::next_account_info(accounts_iter)?;

    let rent_exemption = solana_program::rent::Rent::get()?.minimum_balance(data.len());
    let calculated_pda =
        solana_program::pubkey::Pubkey::find_program_address(&[&account_user.key.to_bytes()], program_id);
    assert_eq!(account_data.key, &calculated_pda.0);
    let bump_seed = calculated_pda.1;

    // 账户不存在，创建并写入数据
    if **account_data.try_borrow_lamports().unwrap() == 0 {
        solana_program::program::invoke_signed(
            &solana_program::system_instruction::create_account(
                account_user.key,
                account_data.key,
                rent_exemption,
                data.len() as u64,
                program_id,
            ),
            accounts,
            &[&[&account_user.key.to_bytes(), &[bump_seed]]],
        )?;
        account_data.data.borrow_mut().copy_from_slice(data);
        return Ok(());
    }

    // 租金补足
    if rent_exemption > account_data.lamports() {
        solana_program::program::invoke(
            &solana_program::system_instruction::transfer(
                account_user.key,
                account_data.key,
                rent_exemption - account_data.lamports(),
            ),
            accounts,
        )?;
    }
    
    // 租金退款
    if rent_exemption < account_data.lamports() {
        **account_user.lamports.borrow_mut() = account_user.lamports() + account_data.lamports() - rent_exemption;
        **account_data.lamports.borrow_mut() = rent_exemption;
    }
    
    // 重新分配空间并写入数据
    account_data.realloc(data.len(), false)?;
    account_data.data.borrow_mut().copy_from_slice(data);

    Ok(())
}
```

## 编译与部署

### 编译

```bash
cargo build-sbf -- -Znext-lockfile-bump
```

编译成功后，在 `target/deploy/` 目录下会生成 `pxsol_ss.so` 文件。

### 部署

```python
import pathlib
import pxsol

pxsol.config.current.log = 1
ada = pxsol.wallet.Wallet(pxsol.core.PriKey.int_decode(0x01))

program_data = pathlib.Path('target/deploy/pxsol_ss.so').read_bytes()
program_pubkey = ada.program_deploy(bytearray(program_data))
print(program_pubkey)  # DVapU9kvtjzFdH3sRd3VDCXjZVkwBR6Cxosx36A5sK5E
```

Solana 部署流程：
1. 创建程序账户
2. 分片上传程序代码（单笔交易最大 1232 字节）
3. 调用 BPF Loader 的 finalize 方法

## 程序交互

### 写入数据

```python
def save(user: pxsol.wallet.Wallet, data: bytearray) -> None:
    prog_pubkey = pxsol.core.PubKey.base58_decode('DVapU9kvtjzFdH3sRd3VDCXjZVkwBR6Cxosx36A5sK5E')
    data_pubkey = prog_pubkey.derive_pda(user.pubkey.p)[0]
    rq = pxsol.core.Requisition(prog_pubkey, [], bytearray())
    rq.account.append(pxsol.core.AccountMeta(user.pubkey, 3))
    rq.account.append(pxsol.core.AccountMeta(data_pubkey, 1))
    rq.account.append(pxsol.core.AccountMeta(pxsol.program.System.pubkey, 0))
    rq.account.append(pxsol.core.AccountMeta(pxsol.program.SysvarRent.pubkey, 0))
    rq.data = data
    tx = pxsol.core.Transaction.requisition_decode(user.pubkey, [rq])
    tx.message.recent_blockhash = pxsol.base58.decode(pxsol.rpc.get_latest_blockhash({})['blockhash'])
    tx.sign([user.prikey])
    txid = pxsol.rpc.send_transaction(base64.b64encode(tx.serialize()).decode(), {})
    pxsol.rpc.wait([txid])
```

### 读取数据

```python
def load(user: pxsol.wallet.Wallet) -> bytearray:
    prog_pubkey = pxsol.core.PubKey.base58_decode('DVapU9kvtjzFdH3sRd3VDCXjZVkwBR6Cxosx36A5sK5E')
    data_pubkey = prog_pubkey.derive_pda(user.pubkey.p)[0]
    info = pxsol.rpc.get_account_info(data_pubkey.base58(), {})
    return base64.b64decode(info['data'][0])
```

## 程序升级

### BPF Loader 演进

| Loader | 地址 | 特点 |
|--------|------|------|
| v1 | BPFLoader1111... | 部署后不可升级 |
| v2 | BPFLoader2111... | 更高效，但不可升级 |
| v3 | BPFLoaderUpgradeab1e... | 当前默认，支持升级 |

### 可升级程序结构

```
Program ID
│
├──> Program Account (主地址，程序入口)
│     └── owner: BPFLoaderUpgradeable
│     └── executable: true
│
└──> ProgramData Account (存储字节码，可变)
      └── .so 字节码
      └── upgrade_authority pubkey
```

### 升级程序

```python
import pathlib
import pxsol

ada = pxsol.wallet.Wallet(pxsol.core.PriKey.int_decode(0x01))
program_pubkey = pxsol.core.PubKey.base58_decode('DVapU9kvtjzFdH3sRd3VDCXjZVkwBR6Cxosx36A5sK5E')
program_data = pathlib.Path('target/deploy/pxsol_ss.so').read_bytes()
ada.program_update(program_pubkey, program_data)
```

## 快速开始

```bash
git clone https://github.com/mohanson/pxsol-ss
cd pxsol-ss

# 部署
python make.py deploy

# 保存数据
python make.py save "The quick brown fox jumps over the lazy dog"

# 读取数据
python make.py load

# 更新数据
python make.py save "片云天共远, 永夜月同孤."
python make.py load
```
