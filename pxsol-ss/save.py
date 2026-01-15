import base64
import pxsol
import sys
import os

# 配置连接到本地验证器
pxsol.config.current.rpc_url = "http://localhost:8899"

# 获取要保存的数据（从命令行参数或默认值）
data_str = sys.argv[1] if len(sys.argv) > 1 else "Hello, Solana!"
data = bytearray(data_str.encode('utf-8'))

# 创建用户钱包（使用固定私钥，方便测试）
user = pxsol.wallet.Wallet(pxsol.core.PriKey.int_decode(0x02))

# 获取程序 ID（优先从文件读取，如果没有则从环境变量或命令行参数）
if os.path.exists('program_id.txt'):
    with open('program_id.txt', 'r') as f:
        program_id = f.read().strip()
elif len(sys.argv) > 2:
    program_id = sys.argv[2]
else:
    program_id = os.getenv('PROGRAM_ID', '你的程序ID')  # 替换为实际程序ID

if program_id == '你的程序ID':
    print("错误：请先运行 deploy.py 部署程序，或设置 PROGRAM_ID 环境变量")
    print("用法: python save.py <数据> [程序ID]")
    sys.exit(1)

prog_pubkey = pxsol.core.PubKey.base58_decode(program_id)

# 派生 PDA 地址
data_pubkey = prog_pubkey.derive_pda(user.pubkey.p)

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
