# 程序的权限说明

## 核心问题

程序拥有哪些账户的权限？

## 权限关系图

```
┌─────────────────────────────────────────────────────────┐
│              程序的权限范围                               │
└─────────────────────────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
        ▼                   ▼                   ▼
┌──────────────┐   ┌──────────────┐   ┌──────────────┐
│ 程序数据账户  │   │ 用户数据账户  │   │ 其他账户      │
│(ProgramData) │   │   (PDA)      │   │              │
├──────────────┤   ├──────────────┤   ├──────────────┤
│ 所有者:      │   │ 所有者:      │   │ 所有者:      │
│ BPF Loader   │   │ 程序本身     │   │ 其他         │
│              │   │ (program_id) │   │              │
│ 程序权限:    │   │              │   │ 程序权限:    │
│ ❌ 不拥有     │   │ ✅ 完全拥有   │   │ ❌ 不拥有     │
└──────────────┘   └──────────────┘   └──────────────┘
```

## 详细说明

### 1. 程序数据账户（ProgramData Account）

#### 对于可升级程序

```
ProgramData Account {
    owner: BPFLoaderUpgradeable,  // 所有者是 BPF Loader
    upgrade_authority: deployer_pubkey,  // 升级权限属于部署者
    data: program_bytecode,  // 存储程序字节码
}
```

**程序拥有的权限**：
- ❌ **不拥有所有权**：ProgramData Account 的所有者是 BPF Loader，不是程序
- ⚠️ **间接控制**：如果是可升级程序，程序可以通过 upgrade_authority 更新代码
- ✅ **可以读取**：程序可以读取自己的代码（如果需要）

**关键点**：
- ProgramData Account 的所有者是 **BPF Loader**，不是程序本身
- 升级权限属于 **部署者**（upgrade_authority），不是程序
- 程序本身不直接拥有 ProgramData Account

#### 对于不可升级程序（你的程序）

```
你的程序（不可升级）
    │
    │ 没有 ProgramData Account
    │
    ▼
程序代码直接存储在 Program Account 中
```

**特点**：
- 没有独立的 ProgramData Account
- 程序代码直接存储在程序账户中
- 程序无法升级

### 2. 用户数据账户（PDA - Program Derived Address）

```
用户数据账户 (PDA) {
    owner: program_id,  // ✅ 所有者是程序本身
    data: user_data,    // 存储用户数据
}
```

**程序拥有的权限**：
- ✅ **完全拥有**：PDA 的所有者是程序（program_id）
- ✅ **可以修改**：程序可以修改 PDA 的数据
- ✅ **可以调整余额**：程序可以调整 PDA 的余额（租金退款）
- ✅ **可以重新分配空间**：程序可以重新分配 PDA 的数据空间

**在你的代码中**：

```rust
// 创建账户时，owner 设置为 program_id
create_account(
    account_user.key,   // 支付者
    account_data.key,   // PDA 地址
    rent_exemption,
    data.len() as u64,
    program_id,        // ✅ owner = program_id（程序拥有）
)

// 程序可以修改 PDA 的数据
account_data.data.borrow_mut().copy_from_slice(data);

// 程序可以调整 PDA 的余额
**account_data.lamports.borrow_mut() = rent_exemption;

// 程序可以重新分配空间
account_data.realloc(data.len(), false)?;
```

### 3. 其他账户

**程序不拥有的账户**：
- ❌ 用户钱包账户：属于用户，程序无法控制
- ❌ System Program：系统程序，程序无法控制
- ❌ Sysvar Rent：系统变量，程序无法控制
- ❌ 其他程序的账户：程序无法控制

## 权限总结表

| 账户类型 | 所有者 | 程序权限 | 说明 |
|---------|--------|---------|------|
| **Program Account** | BPF Loader | ❌ 不拥有 | 程序账户本身属于系统 |
| **ProgramData Account** | BPF Loader | ❌ 不拥有 | 可升级程序的代码存储账户 |
| **用户数据账户 (PDA)** | 程序 (program_id) | ✅ **完全拥有** | 程序创建和管理的用户数据账户 |
| **用户钱包** | 用户 | ❌ 不拥有 | 用户自己的钱包 |
| **系统账户** | 系统 | ❌ 不拥有 | System Program, Sysvar 等 |

## 在你的代码中的实际权限

### 程序拥有的权限

```rust
// ✅ 1. 创建用户数据账户（PDA）
create_account(..., program_id)  // owner = program_id

// ✅ 2. 修改用户数据账户的数据
account_data.data.borrow_mut().copy_from_slice(data);

// ✅ 3. 调整用户数据账户的余额
**account_data.lamports.borrow_mut() = rent_exemption;

// ✅ 4. 重新分配用户数据账户的空间
account_data.realloc(data.len(), false)?;
```

### 程序不拥有的权限

```rust
// ❌ 无法修改程序账户本身（属于 BPF Loader）
// ❌ 无法修改 ProgramData Account（如果存在，属于 BPF Loader）
// ❌ 无法修改用户钱包（属于用户）
// ❌ 无法修改系统账户（属于系统）
```

## 关键理解

### 问题：程序拥有程序数据账户的权限吗？

**答案：不直接拥有**

1. **ProgramData Account 的所有者是 BPF Loader**，不是程序
2. **程序不拥有 ProgramData Account 的所有权**
3. **如果是可升级程序**，升级权限属于部署者（upgrade_authority），程序可以通过升级来间接影响
4. **对于不可升级程序**（你的程序），没有 ProgramData Account

### 问题：程序拥有用户数据账户的权限吗？

**答案：完全拥有**

1. **用户数据账户（PDA）的所有者是程序**（program_id）
2. **程序可以完全控制这些账户**：
   - 修改数据
   - 调整余额
   - 重新分配空间
   - 创建和删除（通过程序逻辑）

## 总结

1. **程序数据账户**：程序不拥有（所有者是 BPF Loader）
2. **用户数据账户（PDA）**：程序完全拥有（所有者是 program_id）
3. **程序的主要权限**：管理自己创建的用户数据账户（PDA）
4. **程序没有的权限**：控制程序账户本身、控制 ProgramData Account
