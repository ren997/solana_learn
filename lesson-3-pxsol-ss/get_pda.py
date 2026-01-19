import pxsol
import os

# 配置连接到本地验证器
pxsol.config.current.rpc_url = "http://localhost:8899"

# 创建用户钱包
user = pxsol.wallet.Wallet(pxsol.core.PriKey.int_decode(0x02))

# 获取程序 ID
if os.path.exists('program_id.txt'):
    with open('program_id.txt', 'r') as f:
        program_id = f.read().strip()
else:
    program_id = input("请输入程序 ID: ")

prog_pubkey = pxsol.core.PubKey.base58_decode(program_id)

# 派生 PDA 地址
data_pubkey = prog_pubkey.derive_pda(user.pubkey.p)

print(f"用户钱包地址: {user.pubkey.base58()}")
print(f"程序 ID: {program_id}")
print(f"PDA 账户地址: {data_pubkey.base58()}")
print(f"\n使用以下命令查看 PDA 账户详情:")
print(f"solana account {data_pubkey.base58()}")
print(f"\n或查看余额:")
print(f"solana balance {data_pubkey.base58()}")
