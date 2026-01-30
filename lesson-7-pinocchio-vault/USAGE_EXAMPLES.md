# Pinocchio Vault 使用示例

## 目录
- [环境准备](#环境准备)
- [构建和部署](#构建和部署)
- [指令示例](#指令示例)
- [完整工作流程](#完整工作流程)
- [故障排除](#故障排除)

## 环境准备

### 1. 安装依赖

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# 验证安装
rustc --version
solana --version
```

### 2. 配置 Solana

```bash
# 设置为本地测试网
solana config set --url localhost

# 创建密钥对 (如果还没有)
solana-keygen new

# 查看配置
solana config get
```

## 构建和部署

### 1. 克隆和构建

```bash
# 进入项目目录
cd solana_learn/lesson-7-pinocchio-vault

# 构建程序
make build

# 或手动构建
cargo build-sbf --release
```

### 2. 启动本地验证器

```bash
# 在新终端启动
make validator

# 或手动启动
solana-test-validator

# 等待验证器启动完成
```

### 3. 部署程序

```bash
# 部署程序
make deploy

# 或手动部署
solana program deploy target/deploy/vault_program.so

# 记录返回的 Program ID
# 例如: Program Id: 7N4HggYEJAtCLJdnHGCtFqfxcB5rhQCsQTze3ftYstVj
```

## 指令示例

### 使用 Solana CLI

#### 1. 初始化金库

```bash
# 设置变量
PROGRAM_ID="你的程序ID"
OWNER=$(solana address)

# 派生 Vault PDA
# 注意: 需要使用工具计算 PDA,这里是示例
VAULT_PDA="计算出的PDA地址"

# 构建指令数据 (discriminator = 0)
INSTRUCTION_DATA="00"

# 发送交易
solana program invoke \
  --program-id $PROGRAM_ID \
  --account $OWNER writable signer \
  --account $VAULT_PDA writable \
  --account 11111111111111111111111111111111 \
  --instruction-data $INSTRUCTION_DATA
```

#### 2. 存款

```bash
# 存款金额 (1 SOL = 1000000000 lamports)
AMOUNT=1000000000  # 1 SOL

# 转换为小端字节 (8 字节)
# discriminator (1 byte) + amount (8 bytes)
INSTRUCTION_DATA="01$(printf '%016x' $AMOUNT | tac -rs ..)"

# 发送交易
solana program invoke \
  --program-id $PROGRAM_ID \
  --account $OWNER writable signer \
  --account $VAULT_PDA writable \
  --account 11111111111111111111111111111111 \
  --instruction-data $INSTRUCTION_DATA
```

#### 3. 取款

```bash
# 取款金额
AMOUNT=500000000  # 0.5 SOL

# 构建指令数据 (discriminator = 2)
INSTRUCTION_DATA="02$(printf '%016x' $AMOUNT | tac -rs ..)"

# 发送交易
solana program invoke \
  --program-id $PROGRAM_ID \
  --account $OWNER writable signer \
  --account $VAULT_PDA writable \
  --account 11111111111111111111111111111111 \
  --instruction-data $INSTRUCTION_DATA
```

### 使用 TypeScript/JavaScript

```typescript
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
} from '@solana/web3.js';

// 连接配置
const connection = new Connection('http://localhost:8899', 'confirmed');
const programId = new PublicKey('你的程序ID');

// 加载密钥对
const payer = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync('~/.config/solana/id.json', 'utf-8')))
);

// 派生 Vault PDA
function deriveVaultPDA(owner: PublicKey, programId: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('vault'), owner.toBuffer()],
    programId
  );
}

// 1. 初始化金库
async function initialize() {
  const [vaultPDA, bump] = deriveVaultPDA(payer.publicKey, programId);
  
  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: vaultPDA, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId,
    data: Buffer.from([0]), // discriminator
  });
  
  const transaction = new Transaction().add(instruction);
  const signature = await connection.sendTransaction(transaction, [payer]);
  await connection.confirmTransaction(signature);
  
  console.log('✓ 金库初始化成功:', signature);
}

// 2. 存款
async function deposit(amount: number) {
  const [vaultPDA, bump] = deriveVaultPDA(payer.publicKey, programId);
  
  const data = Buffer.alloc(9);
  data.writeUInt8(1, 0); // discriminator
  data.writeBigUInt64LE(BigInt(amount), 1); // amount
  
  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: vaultPDA, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId,
    data,
  });
  
  const transaction = new Transaction().add(instruction);
  const signature = await connection.sendTransaction(transaction, [payer]);
  await connection.confirmTransaction(signature);
  
  console.log('✓ 存款成功:', signature);
}

// 3. 取款
async function withdraw(amount: number) {
  const [vaultPDA, bump] = deriveVaultPDA(payer.publicKey, programId);
  
  const data = Buffer.alloc(9);
  data.writeUInt8(2, 0); // discriminator
  data.writeBigUInt64LE(BigInt(amount), 1); // amount
  
  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: vaultPDA, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId,
    data,
  });
  
  const transaction = new Transaction().add(instruction);
  const signature = await connection.sendTransaction(transaction, [payer]);
  await connection.confirmTransaction(signature);
  
  console.log('✓ 取款成功:', signature);
}

// 4. 查询金库余额
async function getVaultBalance() {
  const [vaultPDA, bump] = deriveVaultPDA(payer.publicKey, programId);
  const balance = await connection.getBalance(vaultPDA);
  console.log('金库余额:', balance / 1e9, 'SOL');
  return balance;
}
```

### 使用 Python

```python
from solana.rpc.api import Client
from solana.keypair import Keypair
from solana.publickey import PublicKey
from solana.transaction import Transaction
from solana.system_program import SYS_PROGRAM_ID
from solders.instruction import Instruction, AccountMeta
import struct

# 连接配置
client = Client("http://localhost:8899")
program_id = PublicKey("你的程序ID")

# 加载密钥对
with open("~/.config/solana/id.json", "r") as f:
    secret_key = json.load(f)
payer = Keypair.from_secret_key(bytes(secret_key))

# 派生 Vault PDA
def derive_vault_pda(owner: PublicKey, program_id: PublicKey):
    seeds = [b"vault", bytes(owner)]
    return PublicKey.find_program_address(seeds, program_id)

# 1. 初始化金库
def initialize():
    vault_pda, bump = derive_vault_pda(payer.public_key, program_id)
    
    instruction = Instruction(
        program_id=program_id,
        accounts=[
            AccountMeta(payer.public_key, is_signer=True, is_writable=True),
            AccountMeta(vault_pda, is_signer=False, is_writable=True),
            AccountMeta(SYS_PROGRAM_ID, is_signer=False, is_writable=False),
        ],
        data=bytes([0])  # discriminator
    )
    
    transaction = Transaction().add(instruction)
    response = client.send_transaction(transaction, payer)
    print(f"✓ 金库初始化成功: {response['result']}")

# 2. 存款
def deposit(amount: int):
    vault_pda, bump = derive_vault_pda(payer.public_key, program_id)
    
    # discriminator (1 byte) + amount (8 bytes, little-endian)
    data = struct.pack('<BQ', 1, amount)
    
    instruction = Instruction(
        program_id=program_id,
        accounts=[
            AccountMeta(payer.public_key, is_signer=True, is_writable=True),
            AccountMeta(vault_pda, is_signer=False, is_writable=True),
            AccountMeta(SYS_PROGRAM_ID, is_signer=False, is_writable=False),
        ],
        data=data
    )
    
    transaction = Transaction().add(instruction)
    response = client.send_transaction(transaction, payer)
    print(f"✓ 存款成功: {response['result']}")

# 3. 取款
def withdraw(amount: int):
    vault_pda, bump = derive_vault_pda(payer.public_key, program_id)
    
    # discriminator (1 byte) + amount (8 bytes, little-endian)
    data = struct.pack('<BQ', 2, amount)
    
    instruction = Instruction(
        program_id=program_id,
        accounts=[
            AccountMeta(payer.public_key, is_signer=True, is_writable=True),
            AccountMeta(vault_pda, is_signer=False, is_writable=True),
            AccountMeta(SYS_PROGRAM_ID, is_signer=False, is_writable=False),
        ],
        data=data
    )
    
    transaction = Transaction().add(instruction)
    response = client.send_transaction(transaction, payer)
    print(f"✓ 取款成功: {response['result']}")

# 4. 查询金库余额
def get_vault_balance():
    vault_pda, bump = derive_vault_pda(payer.public_key, program_id)
    balance = client.get_balance(vault_pda)['result']['value']
    print(f"金库余额: {balance / 1e9} SOL")
    return balance
```

## 完整工作流程

### 场景 1: 基本存取款

```bash
# 1. 启动验证器
make validator

# 2. 部署程序
make deploy
# 记录 Program ID

# 3. 初始化金库
# (使用上面的代码示例)

# 4. 存款 1 SOL
# (使用上面的代码示例)

# 5. 查询余额
solana balance <VAULT_PDA>

# 6. 取款 0.5 SOL
# (使用上面的代码示例)

# 7. 再次查询余额
solana balance <VAULT_PDA>
```

### 场景 2: 多次操作

```typescript
async function multipleOperations() {
  // 1. 初始化
  await initialize();
  
  // 2. 多次存款
  await deposit(1_000_000_000); // 1 SOL
  await deposit(500_000_000);   // 0.5 SOL
  await deposit(250_000_000);   // 0.25 SOL
  
  // 3. 查询余额
  const balance = await getVaultBalance();
  console.log('总余额:', balance / 1e9, 'SOL');
  
  // 4. 部分取款
  await withdraw(500_000_000);  // 0.5 SOL
  
  // 5. 最终余额
  await getVaultBalance();
}
```

## 故障排除

### 问题 1: 程序部署失败

```bash
# 检查余额
solana balance

# 如果余额不足,请求空投
solana airdrop 2

# 重新部署
make deploy
```

### 问题 2: 交易失败

```bash
# 查看详细日志
solana logs

# 检查程序日志
make logs

# 使用调试版本
make build-debug
make deploy
```

### 问题 3: PDA 计算错误

```typescript
// 确保使用正确的种子
const [vaultPDA, bump] = PublicKey.findProgramAddressSync(
  [
    Buffer.from('vault'),  // 必须是 'vault'
    owner.toBuffer()       // owner 的公钥
  ],
  programId
);

console.log('Vault PDA:', vaultPDA.toBase58());
console.log('Bump:', bump);
```

### 问题 4: 余额不足

```bash
# 检查账户余额
solana balance

# 检查金库余额
solana balance <VAULT_PDA>

# 确保有足够的 SOL 支付租金和交易费用
```

### 问题 5: 权限错误

```bash
# 确保使用正确的签名者
# 只有金库所有者可以存取款

# 检查当前密钥对
solana address

# 如果需要,切换密钥对
solana config set --keypair /path/to/keypair.json
```

## 调试技巧

### 1. 启用详细日志

```bash
# 构建调试版本
make build-debug

# 部署
make deploy

# 查看日志
make logs
```

### 2. 检查账户状态

```bash
# 查看账户信息
solana account <VAULT_PDA>

# 查看账户数据 (十六进制)
solana account <VAULT_PDA> --output json | jq -r '.account.data[0]'
```

### 3. 模拟交易

```typescript
// 在发送前模拟交易
const simulation = await connection.simulateTransaction(transaction);
console.log('模拟结果:', simulation);

if (simulation.value.err) {
  console.error('交易会失败:', simulation.value.err);
  return;
}

// 如果模拟成功,再发送真实交易
const signature = await connection.sendTransaction(transaction, [payer]);
```

## 最佳实践

1. **始终验证 PDA**: 确保使用正确的种子派生 PDA
2. **检查余额**: 在取款前检查金库余额
3. **处理错误**: 捕获并正确处理所有错误
4. **使用模拟**: 在发送真实交易前先模拟
5. **保留日志**: 记录所有交易签名以便追踪
6. **测试充分**: 在主网部署前充分测试

## 参考资源

- [Solana CLI 文档](https://docs.solana.com/cli)
- [Solana Web3.js 文档](https://solana-labs.github.io/solana-web3.js/)
- [Solana Python SDK](https://michaelhly.github.io/solana-py/)

## 获取帮助

如果遇到问题:
1. 查看 `README.md` 和 `PROJECT_OVERVIEW.md`
2. 检查程序日志 (`make logs`)
3. 阅读错误消息
4. 参考 Solana 官方文档

祝使用愉快! 🚀
