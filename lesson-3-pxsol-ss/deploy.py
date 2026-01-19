import pathlib
import pxsol

# 配置连接到本地验证器
pxsol.config.current.rpc_url = "http://localhost:8899"

# 创建部署者钱包
ada = pxsol.wallet.Wallet(pxsol.core.PriKey.int_decode(0x01))

# 读取程序文件
program_data = pathlib.Path('target/deploy/pxsol_ss.so').read_bytes()

# 部署程序
program_pubkey = ada.program_deploy(bytearray(program_data))
print(f"程序已部署，Program ID: {program_pubkey}")

# 将程序 ID 保存到文件，方便后续使用
with open('program_id.txt', 'w') as f:
    f.write(program_pubkey)
print(f"程序 ID 已保存到 program_id.txt")
