import base64
import pxsol
import os
import sys

# 配置连接到本地验证器
pxsol.config.current.rpc_url = "http://localhost:8899"

# 创建用户钱包（与保存时使用相同的私钥）
user = pxsol.wallet.Wallet(pxsol.core.PriKey.int_decode(0x02))

# 获取程序 ID（优先从文件读取，如果没有则从环境变量或命令行参数）
if os.path.exists('program_id.txt'):
    with open('program_id.txt', 'r') as f:
        program_id = f.read().strip()
elif len(sys.argv) > 1:
    program_id = sys.argv[1]
else:
    program_id = os.getenv('PROGRAM_ID', '你的程序ID')  # 替换为实际程序ID

if program_id == '你的程序ID':
    print("错误：请先运行 deploy.py 部署程序，或设置 PROGRAM_ID 环境变量")
    print("用法: python load.py [程序ID]")
    sys.exit(1)

prog_pubkey = pxsol.core.PubKey.base58_decode(program_id)

# 派生 PDA 地址
data_pubkey = prog_pubkey.derive_pda(user.pubkey.p)

# 读取账户信息
info = pxsol.rpc.get_account_info(data_pubkey.base58(), {})

if info and 'data' in info:
    data_bytes = base64.b64decode(info['data'][0])
    data_str = data_bytes.decode('utf-8')
    print(f"读取的数据: {data_str}")
else:
    print("账户不存在或没有数据")
