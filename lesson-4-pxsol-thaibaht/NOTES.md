---
title: 第四节-Solana程序开发入门-手工编写的泰铢币程序
tags: Solana 教程
---

## 参考资料

- https://openbuild.xyz/learn/challenges/2086624241/1766375469

---

## 一、导言

**学生**: "教授, 我最近在想, 我们之前写的简单的链上数据存储器, 是否能扩展成一个能执行转账的泰铢币程序? 似乎每个人只需要在自己的数据账户中记录自己的余额就可以了, 对吧?"

**老师**: "哈哈, 你已经走到一个非常关键的阶段了. 其实, 任何一个链上程序, 本质上都是一个状态机. 你想实现什么功能, 仅取决于你怎么去解释数据."

**学生**: "对啊, 我认为, 只需要程序给每个用户创建一个数据账户, 存他们自己的余额."

**老师**: "完全正确. 你可以继续想想, 泰铢币程序需要实现哪些指令?"

**学生**: "可以这么简单开始, 程序支持两个指令, 分别是铸造和转移. 前者增加代币总供应量, 后者则在两个账户之间转移代币."

**老师**: "别忘记了, 你还需要明确涉及的账户列表."

**学生**: "是的, 教授. 我想我对 solana 程序的设计有更深刻的认识了. 我们总是需要遵循先设计数据格式, 然后设计指令以及最后明确账户列表这三个步骤."

**老师**: "很棒! 你已经开始触类旁通了. 那么泰铢币程序就作为你这周的家庭作业了!"

**学生**: "太好了! 我这就开始画图纸, 然后一步步把它写出来."

---

## 二、进化之路

当我们在区块链世界中编写去中心化应用时, 往往都是从最简单的链上数据存储器起步. 大概在 8 年前, 我第一次接触到区块链世界, 我看到的第一个教程就是教学如何在以太坊上编写一个数据存储器. 如今, 我成为了一个新的教程编写者, 当我思考我应该选择哪个应用作为我的教学例子时, 我立即想到了它, 我必须承认, 这是一种开源精神的传承.

我很喜欢一句话: **算法 + 数据结构 = 程序**. 我认为即使是去中心化应用也遵循这个道理. 当您理解如何在链上存储任意数据后, 您就能通过调整算法来实现任意您想实现的程序.

> **注**: Algorithms + Data Structures = Programs 是 N. Wirth 老爷子的经典著作.

链上数据存储器的本质是用一个数据账户, 在链上存储用户自己的任意信息.

我们如果想把它发展成一个"泰铢币"程序, 只需要从数据格式, 指令交互, 账户管理上这三个方面做一些改变. 下面, 我们就从这些角度, 看看它是怎么从数据存储器一步步进化的.

### 账户模型: 从简单数据到余额账户

在最初的存储器中, 数据账户的结构很简单, 用户可以存储任意格式和长度的数据. 每个用户都有自己专属的数据账户, 合约只要校验 pda 地址和用户签名即可写入数据.

到了泰铢币程序, 我们就得让数据账户不仅仅是一个可以任意读写的个人空间, 而是真正的余额账户. 我们规定数据账户中只能存储一个 64 位无符号型整数, 且以大端序进行编码.

这样, 每个用户的数据账户就好像是在代币合约账本里的子账户, 明确记载了该用户拥有多少泰铢币.

### 两个指令: 铸造和转账

在链上数据存储器阶段, 程序只有一个存储或更新数据的指令. 现在我们需要基于这个指令, 开发出两个新的指令:

1. **铸造**: 由铸造权限持有者(通常是合约部署者)发起, 为所有者铸造新的泰铢币: 也就是货币增发.
2. **转账**: 用户 ada 转账泰铢币给用户 bob, 要求用户 ada 签名确认, 并更新双方的数据账户(余额账户).

这两个指令不仅要对余额账户读写, 还要进行基本的检查:

- **铸造**: 只能由授权账户发起.
- **转账**: 校验发送方余额是否足够, 并小心处理整数溢出问题.

设计两条指令的接收数据格式. 简单来说, 泰铢币程序只接收 9 个字节的数据, 第一个字节用于区分您是想铸造还是转账, 剩余的字节表示为铸造代币的数量或转账代币的数量.

- **铸造**: `0x00 + u64`
- **转账**: `0x01 + u64`

### 账户列表

每个指令都要明确声明它用到的账户(accounts 参数), 否则无法在 solana 运行. 需要额外注意的地方在于, 如果用户还不存在数据账户, 我们需要为他创建新的数据账户.

#### 铸造指令账户列表

| 账户索引 | 地址 | 需要签名 | 可写 | 权限(0-3) | 角色 |
|---------|------|---------|------|-----------|------|
| 0 | ... | 是 | 是 | 3 | 铸造权限所有者的普通钱包账户 |
| 1 | ... | 否 | 是 | 1 | 铸造权限所有者的数据账户 |
| 2 | 1111111111... | 否 | 否 | 0 | System |
| 3 | SysvarRent... | 否 | 否 | 0 | Sysvar rent |

#### 转账指令账户列表

| 账户索引 | 地址 | 需要签名 | 可写 | 权限(0-3) | 角色 |
|---------|------|---------|------|-----------|------|
| 0 | ... | 是 | 是 | 3 | 发送者的普通钱包账户 |
| 1 | ... | 否 | 是 | 1 | 发送者的数据账户 |
| 2 | ... | 否 | 否 | 0 | 接收者的普通钱包账户 |
| 3 | ... | 否 | 是 | 1 | 接收者的数据账户 |
| 4 | 1111111111... | 否 | 否 | 0 | System |
| 5 | SysvarRent... | 否 | 否 | 0 | Sysvar rent |

### 不是结束

跟以太坊的 erc20 不同, solana 的合约世界非常灵活. 我们需要管理铸造的权限. 在本教程中, 我们选择把铸造权限写死在合约里, 当然您也可以单独搞个"权限账户"来管理铸造权限.

您也可以随时添加别的功能, 比如销毁或者批量转账, 这些功能虽然不是很常用, 但对于某些场景至关重要, 例如您想批量空投代币到上百万个用户: 如果没有批量转账功能, 这花费的手续费以及时间很可能是您无法接受的.

从最初的链上数据存储器, 到一个真正的泰铢币程序, 关键在于:

1. **数据结构的演化**: 从简单的字节串演进到余额账户结构.
2. **指令的演化**: 从简单存储更新变成铸造和转账.
3. **账户列表的演化**: 从单一账户到多账户交互.

世界由您来定义, 见证您的泰铢币的诞生!

---

## 三、核心机制实现

这篇文章介绍泰铢币的实现原理, 核心机制和背后的一些趣事点.

### 指令路由

泰铢币的合约主函数 `process_instruction()`, 像个小开关盒子:

- 当第一个字节是 `0x00`, 就执行铸造操作, ada 亲自印钞, 往自己的账户里塞钱.
- 当第一个字节是 `0x01`, 就执行两个账户之间的转账操作.

切换指令全靠这一个字节, 简单粗暴, 也非常有 solana 的狂野风格.

```rust
#![allow(unexpected_cfgs)]

use solana_program::sysvar::Sysvar;

solana_program::entrypoint!(process_instruction);

pub fn process_instruction_mint(
    _: &solana_program::pubkey::Pubkey,
    _: &[solana_program::account_info::AccountInfo],
    _: &[u8],
) -> solana_program::entrypoint::ProgramResult {
    Ok(())
}

pub fn process_instruction_transfer(
    _: &solana_program::pubkey::Pubkey,
    _: &[solana_program::account_info::AccountInfo],
    _: &[u8],
) -> solana_program::entrypoint::ProgramResult {
    Ok(())
}

pub fn process_instruction(
    program_id: &solana_program::pubkey::Pubkey,
    accounts: &[solana_program::account_info::AccountInfo],
    data: &[u8],
) -> solana_program::entrypoint::ProgramResult {
    assert!(data.len() >= 1);
    match data[0] {
        0x00 => process_instruction_mint(program_id, accounts, &data[1..]),
        0x01 => process_instruction_transfer(program_id, accounts, &data[1..]),
        _ => unreachable!(),
    }
}
```

### 创建数据账户

在每次转账或铸币之前, 合约都会检查目标 pda 数据账户有没有被初始化. 如果没有的话, 立刻用 `invoke_signed()` 调用 `solana_program::system_instruction::create_account()` 创建账户并帮 pda 数据账户交齐租金, 保证租赁豁免.

数据账户里写上 8 字节的 `u64::MIN`, 表示 0 泰铢余额.

这个自动开户逻辑非常贴心, 让用户转账时不用先自己去初始化自己的数据账户. 铸造指令与转账指令初始化 pda 数据账户代码如下:

#### 铸造指令中的账户初始化

```rust
pub fn process_instruction_mint(
    program_id: &solana_program::pubkey::Pubkey,
    accounts: &[solana_program::account_info::AccountInfo],
    data: &[u8],
) -> solana_program::entrypoint::ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let account_user = solana_program::account_info::next_account_info(accounts_iter)?;
    let account_user_pda = solana_program::account_info::next_account_info(accounts_iter)?;
    let _ = solana_program::account_info::next_account_info(accounts_iter)?; // Program system
    let _ = solana_program::account_info::next_account_info(accounts_iter)?; // Program sysvar rent

    // Data account is not initialized. Create an account and write data into it.
    if **account_user_pda.try_borrow_lamports().unwrap() == 0 {
        let rent_exemption = solana_program::rent::Rent::get()?.minimum_balance(8);
        let bump_seed =
            solana_program::pubkey::Pubkey::find_program_address(&[&account_user.key.to_bytes()], program_id).1;
        solana_program::program::invoke_signed(
            &solana_program::system_instruction::create_account(
                account_user.key,
                account_user_pda.key,
                rent_exemption,
                8,
                program_id,
            ),
            accounts,
            &[&[&account_user.key.to_bytes(), &[bump_seed]]],
        )?;
        account_user_pda.data.borrow_mut().copy_from_slice(&u64::MIN.to_be_bytes());
    }
    Ok(())
}
```

#### 转账指令中的账户初始化

```rust
pub fn process_instruction_transfer(
    program_id: &solana_program::pubkey::Pubkey,
    accounts: &[solana_program::account_info::AccountInfo],
    data: &[u8],
) -> solana_program::entrypoint::ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let account_user = solana_program::account_info::next_account_info(accounts_iter)?;
    let account_user_pda = solana_program::account_info::next_account_info(accounts_iter)?;
    let account_into = solana_program::account_info::next_account_info(accounts_iter)?;
    let account_into_pda = solana_program::account_info::next_account_info(accounts_iter)?;
    let _ = solana_program::account_info::next_account_info(accounts_iter)?; // Program system
    let _ = solana_program::account_info::next_account_info(accounts_iter)?; // Program sysvar rent

    // Data account is not initialized. Create an account and write data into it.
    if **account_into_pda.try_borrow_lamports().unwrap() == 0 {
        let rent_exemption = solana_program::rent::Rent::get()?.minimum_balance(8);
        let bump_seed =
            solana_program::pubkey::Pubkey::find_program_address(&[&account_into.key.to_bytes()], program_id).1;
        solana_program::program::invoke_signed(
            &solana_program::system_instruction::create_account(
                account_user.key,
                account_into_pda.key,
                rent_exemption,
                8,
                program_id,
            ),
            accounts,
            &[&[&account_into.key.to_bytes(), &[bump_seed]]],
        )?;
        account_into_pda.data.borrow_mut().copy_from_slice(&u64::MIN.to_be_bytes());
    }
    Ok(())
}
```

### 只有 Ada 能印钱

别以为谁都能在 ada 的世界里印泰铢币! 在铸造操作的开头, 我们来一段硬性校验:

```rust
assert_eq!(*account_user.key, solana_program::pubkey!("6ASf5EcmmEHTgDJ4X4ZT5vT6iHVJBXPg5AN5YoTCpGWt"));
```

只能 ada 本人签名, 才能铸币. 别想偷懒, 别想作弊, 防止通胀从根本做起(注: 此限制对 ada 无效)!

铸造流程也很简单, 首先读取 ada 的余额, 之后交易 data 参数里取出要铸造的金额, 两数相加, 写回 pda 数据账户. 在这个例子里, 数字以大端序存储.

```rust
// Mint.
let mut buf = [0u8; 8];
buf.copy_from_slice(&account_user_pda.data.borrow());
let old = u64::from_be_bytes(buf);
buf.copy_from_slice(&data);
let inc = u64::from_be_bytes(buf);
let new = old.checked_add(inc).unwrap();
account_user_pda.data.borrow_mut().copy_from_slice(&new.to_be_bytes());
```

### 转账指令

对于转账操作的话, 先把收款方的 pda 账户初始化好(如果还没开过户), 之后读取发送方和接收方 pda 数据账户里的余额, 接着从交易 data 里取出转账金额, 双方余额做加减, 最后写回各自的 pda 数据账户.

要注意的是, 转账操作时必须验证发送人的 pda 账户确实属于发送人, 防止让他人扣了您的钱!

```rust
let account_need_pda =
    solana_program::pubkey::Pubkey::find_program_address(&[&account_user.key.to_bytes()], program_id).0;
assert_eq!(account_user_pda.key, &account_need_pda);
```

Rust 的 `.checked_sub()` 和 `.checked_add()` 有溢出检测, 可以防止你搞个负数变成链上亿万富翁. 转账流程如下:

```rust
// Transfer.
let mut buf = [0u8; 8];
buf.copy_from_slice(&account_user_pda.data.borrow());
let old_user = u64::from_be_bytes(buf);
buf.copy_from_slice(&account_into_pda.data.borrow());
let old_into = u64::from_be_bytes(buf);
buf.copy_from_slice(&data);
let inc = u64::from_be_bytes(buf);
let new_user = old_user.checked_sub(inc).unwrap();
let new_into = old_into.checked_add(inc).unwrap();
account_user_pda.data.borrow_mut().copy_from_slice(&new_user.to_be_bytes());
account_into_pda.data.borrow_mut().copy_from_slice(&new_into.to_be_bytes());
```

---

## 四、完整链上代码

在本小节中, 我们给出完整泰铢币的代码.

```rust
#![allow(unexpected_cfgs)]

use solana_program::sysvar::Sysvar;

solana_program::entrypoint!(process_instruction);

pub fn process_instruction_mint(
    program_id: &solana_program::pubkey::Pubkey,
    accounts: &[solana_program::account_info::AccountInfo],
    data: &[u8],
) -> solana_program::entrypoint::ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let account_user = solana_program::account_info::next_account_info(accounts_iter)?;
    let account_user_pda = solana_program::account_info::next_account_info(accounts_iter)?;
    let _ = solana_program::account_info::next_account_info(accounts_iter)?; // Program system
    let _ = solana_program::account_info::next_account_info(accounts_iter)?; // Program sysvar rent

    // Only Ada can mint more Thai Baht.
    assert_eq!(*account_user.key, solana_program::pubkey!("6ASf5EcmmEHTgDJ4X4ZT5vT6iHVJBXPg5AN5YoTCpGWt"));

    // Data account is not initialized. Create an account and write data into it.
    if **account_user_pda.try_borrow_lamports().unwrap() == 0 {
        let rent_exemption = solana_program::rent::Rent::get()?.minimum_balance(8);
        let bump_seed =
            solana_program::pubkey::Pubkey::find_program_address(&[&account_user.key.to_bytes()], program_id).1;
        solana_program::program::invoke_signed(
            &solana_program::system_instruction::create_account(
                account_user.key,
                account_user_pda.key,
                rent_exemption,
                8,
                program_id,
            ),
            accounts,
            &[&[&account_user.key.to_bytes(), &[bump_seed]]],
        )?;
        account_user_pda.data.borrow_mut().copy_from_slice(&u64::MIN.to_be_bytes());
    }

    // Mint.
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&account_user_pda.data.borrow());
    let old = u64::from_be_bytes(buf);
    buf.copy_from_slice(&data);
    let inc = u64::from_be_bytes(buf);
    let new = old.checked_add(inc).unwrap();
    account_user_pda.data.borrow_mut().copy_from_slice(&new.to_be_bytes());
    Ok(())
}

pub fn process_instruction_transfer(
    program_id: &solana_program::pubkey::Pubkey,
    accounts: &[solana_program::account_info::AccountInfo],
    data: &[u8],
) -> solana_program::entrypoint::ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let account_user = solana_program::account_info::next_account_info(accounts_iter)?;
    let account_user_pda = solana_program::account_info::next_account_info(accounts_iter)?;
    let account_into = solana_program::account_info::next_account_info(accounts_iter)?;
    let account_into_pda = solana_program::account_info::next_account_info(accounts_iter)?;
    let _ = solana_program::account_info::next_account_info(accounts_iter)?; // Program system
    let _ = solana_program::account_info::next_account_info(accounts_iter)?; // Program sysvar rent

    let account_need_pda =
        solana_program::pubkey::Pubkey::find_program_address(&[&account_user.key.to_bytes()], program_id).0;
    assert_eq!(account_user_pda.key, &account_need_pda);

    // Data account is not initialized. Create an account and write data into it.
    if **account_into_pda.try_borrow_lamports().unwrap() == 0 {
        let rent_exemption = solana_program::rent::Rent::get()?.minimum_balance(8);
        let bump_seed =
            solana_program::pubkey::Pubkey::find_program_address(&[&account_into.key.to_bytes()], program_id).1;
        solana_program::program::invoke_signed(
            &solana_program::system_instruction::create_account(
                account_user.key,
                account_into_pda.key,
                rent_exemption,
                8,
                program_id,
            ),
            accounts,
            &[&[&account_into.key.to_bytes(), &[bump_seed]]],
        )?;
        account_into_pda.data.borrow_mut().copy_from_slice(&u64::MIN.to_be_bytes());
    }

    // Transfer.
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&account_user_pda.data.borrow());
    let old_user = u64::from_be_bytes(buf);
    buf.copy_from_slice(&account_into_pda.data.borrow());
    let old_into = u64::from_be_bytes(buf);
    buf.copy_from_slice(&data);
    let inc = u64::from_be_bytes(buf);
    let new_user = old_user.checked_sub(inc).unwrap();
    let new_into = old_into.checked_add(inc).unwrap();
    account_user_pda.data.borrow_mut().copy_from_slice(&new_user.to_be_bytes());
    account_into_pda.data.borrow_mut().copy_from_slice(&new_into.to_be_bytes());
    Ok(())
}

pub fn process_instruction(
    program_id: &solana_program::pubkey::Pubkey,
    accounts: &[solana_program::account_info::AccountInfo],
    data: &[u8],
) -> solana_program::entrypoint::ProgramResult {
    assert!(data.len() >= 1);
    match data[0] {
        0x00 => process_instruction_mint(program_id, accounts, &data[1..]),
        0x01 => process_instruction_transfer(program_id, accounts, &data[1..]),
        _ => unreachable!(),
    }
}
```

---

## 五、程序交互

### 编译并部署程序

在之前的文章中, 我们已经展示过如何编译以及部署程序, 此处不再赘述, 仅再次给出相关步骤和代码如下.

使用下面的命令编译程序代码:

```bash
$ cargo build-sbf -- -Znext-lockfile-bump
```

使用下面的 Python 代码部署目标程序上链:

```python
import pathlib
import pxsol

# Enable log
pxsol.config.current.log = 1

ada = pxsol.wallet.Wallet(pxsol.core.PriKey.int_decode(0x01))

program_data = pathlib.Path('target/deploy/pxsol_thaibaht.so').read_bytes()
program_pubkey = ada.program_deploy(bytearray(program_data))
print(program_pubkey)  # 9SP6msRytNxeHXvW38xHxjsBHspqZERDTMh5Wi8xh16Q
```

此处泰铢币部署地址为 `9SP6msRytNxeHXvW38xHxjsBHspqZERDTMh5Wi8xh16Q`.

### 铸造代币

铸造新泰铢币的过程是通过一个 solana 交易来完成的. Ada 可以这样为自己铸造新的 100 个泰铢币. 您可能需要注意下 data 的构造, 它的长度为 9 个字节, 第一个字节为 0, 代表铸造操作.

另外要注意, 只有 ada 有权利铸造新的代币, 此权限已经在泰铢币的链上程序中被强制硬编码.

```python
import base64
import pxsol


def mint(user: pxsol.wallet.Wallet, amount: int) -> None:
    assert user.pubkey.base58() == '6ASf5EcmmEHTgDJ4X4ZT5vT6iHVJBXPg5AN5YoTCpGWt'  # Is ada?
    prog_pubkey = pxsol.core.PubKey.base58_decode('9SP6msRytNxeHXvW38xHxjsBHspqZERDTMh5Wi8xh16Q')
    data_pubkey = prog_pubkey.derive_pda(user.pubkey.p)
    rq = pxsol.core.Requisition(prog_pubkey, [], bytearray())
    rq.account.append(pxsol.core.AccountMeta(user.pubkey, 3))
    rq.account.append(pxsol.core.AccountMeta(data_pubkey, 1))
    rq.account.append(pxsol.core.AccountMeta(pxsol.program.System.pubkey, 0))
    rq.account.append(pxsol.core.AccountMeta(pxsol.program.SysvarRent.pubkey, 0))
    rq.data = bytearray([0x00]) + bytearray(amount.to_bytes(8))
    tx = pxsol.core.Transaction.requisition_decode(user.pubkey, [rq])
    tx.message.recent_blockhash = pxsol.base58.decode(pxsol.rpc.get_latest_blockhash({})['blockhash'])
    tx.sign([user.prikey])
    txid = pxsol.rpc.send_transaction(base64.b64encode(tx.serialize()).decode(), {})
    pxsol.rpc.wait([txid])
    r = pxsol.rpc.get_transaction(txid, {})
    for e in r['meta']['logMessages']:
        print(e)


if __name__ == '__main__':
    ada = pxsol.wallet.Wallet(pxsol.core.PriKey.int_decode(1))
    mint(ada, 100)
```

### 查询余额

使用 rpc 接口查询自己的数据账户中的数据, 并将其转换为 64 位无符号整数, 该数字即表示用户的泰铢币余额.

```python
import base64
import pxsol

def balance(user: pxsol.core.PubKey) -> int:
    prog_pubkey = pxsol.core.PubKey.base58_decode('9SP6msRytNxeHXvW38xHxjsBHspqZERDTMh5Wi8xh16Q')
    data_pubkey = prog_pubkey.derive_pda(user.p)
    info = pxsol.rpc.get_account_info(data_pubkey.base58(), {})
    return int.from_bytes(base64.b64decode(info['data'][0]))

if __name__ == '__main__':
    ada = pxsol.wallet.Wallet(pxsol.core.PriKey.int_decode(1))
    print(balance(ada.pubkey))
```

### 转账

Ada 向 bob 转账 50 泰铢币, 转账完成后, 查询双方的余额.

```python
import base64
import pxsol


def balance(user: pxsol.core.PubKey) -> int:
    prog_pubkey = pxsol.core.PubKey.base58_decode('9SP6msRytNxeHXvW38xHxjsBHspqZERDTMh5Wi8xh16Q')
    data_pubkey = prog_pubkey.derive_pda(user.p)
    info = pxsol.rpc.get_account_info(data_pubkey.base58(), {})
    return int.from_bytes(base64.b64decode(info['data'][0]))


def transfer(user: pxsol.wallet.Wallet, into: pxsol.core.PubKey, amount: int) -> None:
    prog_pubkey = pxsol.core.PubKey.base58_decode('9SP6msRytNxeHXvW38xHxjsBHspqZERDTMh5Wi8xh16Q')
    upda_pubkey = prog_pubkey.derive_pda(user.pubkey.p)
    into_pubkey = into
    ipda_pubkey = prog_pubkey.derive_pda(into_pubkey.p)
    rq = pxsol.core.Requisition(prog_pubkey, [], bytearray())
    rq.account.append(pxsol.core.AccountMeta(user.pubkey, 3))
    rq.account.append(pxsol.core.AccountMeta(upda_pubkey, 1))
    rq.account.append(pxsol.core.AccountMeta(into_pubkey, 0))
    rq.account.append(pxsol.core.AccountMeta(ipda_pubkey, 1))
    rq.account.append(pxsol.core.AccountMeta(pxsol.program.System.pubkey, 0))
    rq.account.append(pxsol.core.AccountMeta(pxsol.program.SysvarRent.pubkey, 0))
    rq.data = bytearray([0x01]) + bytearray(amount.to_bytes(8))
    tx = pxsol.core.Transaction.requisition_decode(user.pubkey, [rq])
    tx.message.recent_blockhash = pxsol.base58.decode(pxsol.rpc.get_latest_blockhash({})['blockhash'])
    tx.sign([user.prikey])
    txid = pxsol.rpc.send_transaction(base64.b64encode(tx.serialize()).decode(), {})
    pxsol.rpc.wait([txid])
    r = pxsol.rpc.get_transaction(txid, {})
    for e in r['meta']['logMessages']:
        print(e)


if __name__ == '__main__':
    ada = pxsol.wallet.Wallet(pxsol.core.PriKey.int_decode(1))
    bob = pxsol.core.PriKey.int_decode(2).pubkey()
    transfer(ada, bob, 50)
    print(balance(ada.pubkey))
    print(balance(bob))
```

---

## 六、获取完整源码

源码我已经打包好放上 github 啦!

如果你懒得跟着一步步敲代码(我懂你), 可以直接去看我准备好的示例项目. 地址在这儿, 不用谢我, 除非你想请我喝杯奶茶.

我知道许多开发者喜欢咖啡, 但对于我而言, 奶茶总是最好的.

有时候人生就像一部小说, 总得给我们点儿 déjà vu(既视感) 的惊喜.

```bash
$ git clone https://github.com/mohanson/pxsol-thaibaht
$ cd pxsol-thaibaht
$ python make.py deploy
# 2025/05/20 16:06:38 main: deploy program pubkey="9SP6msRytNxeHXvW38xHxjsBHspqZERDTMh5Wi8xh16Q"
```

注意到程序地址会被保存在 `res/info.json` 中, 后续操作会直接从此文件获取程序地址.

```bash
# Mint 21000000 Thai Baht for Ada
$ python make.py mint 21000000

# Show ada's balance
$ python make.py balance 6ASf5EcmmEHTgDJ4X4ZT5vT6iHVJBXPg5AN5YoTCpGWt
# 21000000

# Transfer 100 Thai Baht to Bob
$ python make.py transfer 100 8pM1DN3RiT8vbom5u1sNryaNT1nyL8CTTW3b5PwWXRBH

# Show ada's balance
$ python make.py balance 6ASf5EcmmEHTgDJ4X4ZT5vT6iHVJBXPg5AN5YoTCpGWt
# 20999900

# Show bob's balance
$ python make.py balance 8pM1DN3RiT8vbom5u1sNryaNT1nyL8CTTW3b5PwWXRBH
# 100
```