# 程序测试指南

## 测试环境准备

### 1. 编译程序

```bash
cd /home/dmin/workspace/solana_learn/pxsol-ss

# 编译 Solana 程序
cargo-build-sbf -- -Znext-lockfile-bump

# 编译成功后，会在 target/deploy/pxsol_ss.so 生成程序文件
```

### 2. 启动本地验证器

在**终端 1** 启动本地 Solana 验证器：

```bash
# 启动本地验证器（会占用终端）
solana-test-validator

# 或者使用安静模式（推荐）
solana-test-validator --reset --quiet
```

**说明**：
- `--reset`：每次启动时重置数据（清空之前的状态）
- `--quiet`：减少输出信息
- 验证器会一直运行，不要关闭这个终端
- **这个终端只用于运行验证器，不要在这里执行其他命令**

### 3. 配置 CLI 连接到本地网络

在**终端 2** 配置 Solana CLI（**后续所有操作都在终端 2 执行**）：

```bash
# 设置连接到本地验证器
solana config set --url localhost

# 验证配置
solana config get

# 应该看到：
# Config File: /home/dmin/.config/solana/cli/config.yml
# RPC URL: http://localhost:8899
# WebSocket URL: ws://localhost:8900
```

### 4. 创建测试钱包（如果还没有）

```bash
# 创建新的密钥对
solana-keygen new --outfile ~/.config/solana/id.json

# 或者使用默认位置
solana-keygen new

# 检查钱包地址和余额
solana address
solana balance
```

## 方法一：使用 Solana CLI 部署和测试

**注意：以下所有命令都在终端 2 执行**

### 步骤 1：部署程序

```bash
# 部署程序到本地验证器
solana program deploy target/deploy/pxsol_ss.so

# 部署成功后会显示：
# Program Id: <你的程序地址>
# 例如：Program Id: DVapU9kvtjzFdH3sRd3VDCXjZVkwBR6Cxosx36A5sK5E
```

**重要**：记住这个 Program Id，后续测试需要用到。

### 步骤 2：验证程序已部署

```bash
# 查看程序信息
solana program show <你的程序ID>

# 应该显示程序的详细信息
```

## 方法二：使用 Python + pxsol 库测试（推荐）

**注意：以下所有操作都在终端 2 执行**

### 前置条件

需要安装 `pxsol` Python 库：

```bash
pip install pxsol
```

### 步骤 1：部署程序

创建 `deploy.py`：

```python
import pathlib
import pxsol

# 配置连接到本地验证器
pxsol.rpc.set_url("http://localhost:8899")

# 创建部署者钱包
ada = pxsol.wallet.Wallet(pxsol.core.PriKey.int_decode(0x01))

# 读取程序文件
program_data = pathlib.Path('target/deploy/pxsol_ss.so').read_bytes()

# 部署程序
program_pubkey = ada.program_deploy(bytearray(program_data))
print(f"程序已部署，Program ID: {program_pubkey}")
```

运行部署：

```bash
python deploy.py
```

### 步骤 2：保存数据

创建 `save.py`：

```python
import base64
import pxsol
import sys

# 配置连接到本地验证器
pxsol.rpc.set_url("http://localhost:8899")

# 获取要保存的数据（从命令行参数或默认值）
data_str = sys.argv[1] if len(sys.argv) > 1 else "Hello, Solana!"
data = bytearray(data_str.encode('utf-8'))

# 创建用户钱包（使用固定私钥，方便测试）
user = pxsol.wallet.Wallet(pxsol.core.PriKey.int_decode(0x02))

# 程序 ID（从部署时获取）
prog_pubkey = pxsol.core.PubKey.base58_decode('你的程序ID')  # 替换为实际程序ID

# 派生 PDA 地址
data_pubkey = prog_pubkey.derive_pda(user.pubkey.p)[0]

# 构建指令
rq = pxsol.core.Requisition(prog_pubkey, [], bytearray())
rq.account.append(pxsol.core.AccountMeta(user.pubkey, 3))  # 用户钱包（签名、可写）
rq.account.append(pxsol.core.AccountMeta(data_pubkey, 1))  # PDA 账户（可写）
rq.account.append(pxsol.core.AccountMeta(pxsol.program.System.pubkey, 0))  # System Program
rq.account.append(pxsol.core.AccountMeta(pxsol.program.SysvarRent.pubkey, 0))  # Sysvar Rent
rq.data = data

# 构建并发送交易
tx = pxsol.core.Transaction.requisition_decode(user.pubkey, [rq])
tx.message.recent_blockhash = pxsol.base58.decode(pxsol.rpc.get_latest_blockhash({})['blockhash'])
tx.sign([user.prikey])
txid = pxsol.rpc.send_transaction(base64.b64encode(tx.serialize()).decode(), {})
pxsol.rpc.wait([txid])

print(f"数据已保存: {data_str}")
print(f"交易 ID: {txid}")
```

运行保存：

```bash
python save.py "The quick brown fox jumps over the lazy dog"
```

### 步骤 3：读取数据

创建 `load.py`：

```python
import base64
import pxsol

# 配置连接到本地验证器
pxsol.rpc.set_url("http://localhost:8899")

# 创建用户钱包（与保存时使用相同的私钥）
user = pxsol.wallet.Wallet(pxsol.core.PriKey.int_decode(0x02))

# 程序 ID
prog_pubkey = pxsol.core.PubKey.base58_decode('你的程序ID')  # 替换为实际程序ID

# 派生 PDA 地址
data_pubkey = prog_pubkey.derive_pda(user.pubkey.p)[0]

# 读取账户信息
info = pxsol.rpc.get_account_info(data_pubkey.base58(), {})

if info and 'data' in info:
    data_bytes = base64.b64decode(info['data'][0])
    data_str = data_bytes.decode('utf-8')
    print(f"读取的数据: {data_str}")
else:
    print("账户不存在或没有数据")
```

运行读取：

```bash
python load.py
```

### 步骤 4：更新数据

```bash
# 保存新数据
python save.py "片云天共远, 永夜月同孤."

# 读取验证
python load.py
```

## 方法三：使用 Solana CLI + JavaScript/TypeScript

**注意：以下所有操作都在终端 2 执行**

### 使用 @solana/web3.js

创建 `test.js`：

```javascript
const { Connection, Keypair, PublicKey, SystemProgram, Transaction, sendAndConfirmTransaction } = require('@solana/web3.js');
const { createAccount, createAccountWithSeed } = require('@solana/web3.js');
const fs = require('fs');

// 连接到本地验证器
const connection = new Connection('http://localhost:8899', 'confirmed');

// 创建用户钱包
const user = Keypair.generate();

// 程序 ID（从部署时获取）
const programId = new PublicKey('你的程序ID');  // 替换为实际程序ID

// 派生 PDA
const [pda, bump] = PublicKey.findProgramAddressSync(
  [user.publicKey.toBuffer()],
  programId
);

// 准备数据
const data = Buffer.from("Hello, Solana!", 'utf-8');

// 构建交易（简化版，实际需要更完整的实现）
// ...
```

## 测试检查清单

### ✅ 基本功能测试

- [ ] 程序编译成功
- [ ] 本地验证器启动成功
- [ ] 程序部署成功
- [ ] 保存数据成功
- [ ] 读取数据成功
- [ ] 更新数据成功

### ✅ 边界情况测试

- [ ] 首次保存数据（创建 PDA）
- [ ] 更新为更长的数据（租金补足）
- [ ] 更新为更短的数据（租金退款）
- [ ] 多次更新数据
- [ ] 不同用户的数据隔离

### ✅ 错误处理测试

- [ ] 未签名的交易（应该失败）
- [ ] 错误的账户顺序（应该失败）
- [ ] 错误的 PDA 地址（应该失败）

## 常见问题

### 1. 验证器启动失败

```bash
# 检查端口是否被占用
lsof -i :8899
lsof -i :8900

# 如果被占用，可以指定其他端口
solana-test-validator --rpc-port 8899 --rpc-bind-address 127.0.0.1
```

### 2. 部署失败

```bash
# 检查钱包余额
solana balance

# 如果余额不足，本地验证器会自动给测试钱包充值
# 或者手动充值
solana airdrop 1
```

### 3. 程序调用失败

- 检查程序 ID 是否正确
- 检查账户顺序是否正确
- 检查用户钱包是否签名
- 检查 PDA 地址是否正确派生

## 快速测试脚本

创建一个 `test.sh` 脚本：

```bash
#!/bin/bash

# 编译
echo "编译程序..."
cargo-build-sbf -- -Znext-lockfile-bump

# 部署（需要先启动验证器）
echo "部署程序..."
PROGRAM_ID=$(solana program deploy target/deploy/pxsol_ss.so --output json | jq -r '.programId')
echo "程序 ID: $PROGRAM_ID"

# 保存数据
echo "保存数据..."
python save.py "Test data"

# 读取数据
echo "读取数据..."
python load.py
```

## 终端使用总结

### 终端 1：运行验证器
- **唯一用途**：运行 `solana-test-validator`
- **不要关闭**：验证器需要一直运行
- **不要执行其他命令**：保持验证器运行即可

### 终端 2：执行所有其他操作
- ✅ 配置 CLI
- ✅ 创建钱包
- ✅ 部署程序
- ✅ 运行 Python 脚本
- ✅ 执行测试命令
- ✅ 查看程序信息

**简单记忆**：
- 终端 1 = 验证器（保持运行）
- 终端 2 = 所有其他操作

## 总结

推荐使用**方法二（Python + pxsol）**，因为：
- 代码简单易懂
- 与 NOTES.md 中的示例一致
- 便于调试和修改

测试流程：
1. **终端 1**：启动验证器
2. **终端 2**：部署程序
3. **终端 2**：保存数据
4. **终端 2**：读取数据
5. **终端 2**：更新数据
6. **终端 2**：验证功能
