import argparse
import base64
import pxsol

# ============ 命令行参数解析 ============
parser = argparse.ArgumentParser()
parser.add_argument('--net', type=str, choices=['develop', 'mainnet', 'testnet'], default='develop')
parser.add_argument('--prikey', type=str, default='11111111111111111111111111111112')  # 默认测试私钥
parser.add_argument('args', nargs='+')  # 命令：init / update / load
args = parser.parse_args()

# ============ 初始化客户端 ============
# 从私钥创建钱包
user = pxsol.wallet.Wallet(pxsol.core.PriKey.base58_decode(args.prikey))

# 程序 ID（从 Anchor.toml 或部署输出获取）
prog_pubkey = pxsol.core.PubKey.base58_decode('5e44g7KZvJuhEPEuYX6S8tHWtb2FEyCg41HvDYwwV7z5')

# 推导 PDA 地址（种子：b'data' + 用户公钥）
data_pubkey = prog_pubkey.derive_pda(b'data' + user.pubkey.p)[0]


def init():
    """初始化账户（创建 PDA）"""
    # 创建指令请求
    rq = pxsol.core.Requisition(prog_pubkey, [], bytearray())
    
    # 添加账户（顺序必须与合约定义一致）
    rq.account.append(pxsol.core.AccountMeta(user.pubkey, 3))              # 0: user (signer + writable)
    rq.account.append(pxsol.core.AccountMeta(data_pubkey, 1))              # 1: user_pda (writable)
    rq.account.append(pxsol.core.AccountMeta(pxsol.program.System.pubkey, 0))  # 2: system_program
    
    # 构造指令数据：只有方法 discriminator
    # [220, 59, 207, 236, 108, 250, 47, 100] = sha256("global:init")[:8]
    rq.data = bytearray().join([
        bytearray([220, 59, 207, 236, 108, 250, 47, 100]),  # init 方法的 discriminator
    ])
    
    # 构造、签名、发送交易
    tx = pxsol.core.Transaction.requisition_decode(user.pubkey, [rq])
    tx.message.recent_blockhash = pxsol.base58.decode(pxsol.rpc.get_latest_blockhash({})['blockhash'])
    tx.sign([user.prikey])
    txid = pxsol.rpc.send_transaction(base64.b64encode(tx.serialize()).decode(), {})
    
    # 等待交易确认并打印日志
    pxsol.rpc.wait([txid])
    r = pxsol.rpc.get_transaction(txid, {})
    for e in r['meta']['logMessages']:
        print(e)


def update():
    """更新账户数据"""
    # 创建指令请求
    rq = pxsol.core.Requisition(prog_pubkey, [], bytearray())
    
    # 添加账户（顺序与 init 相同）
    rq.account.append(pxsol.core.AccountMeta(user.pubkey, 3))              # 0: user
    rq.account.append(pxsol.core.AccountMeta(data_pubkey, 1))              # 1: user_pda
    rq.account.append(pxsol.core.AccountMeta(pxsol.program.System.pubkey, 0))  # 2: system_program
    
    # 构造指令数据：discriminator + 参数
    # [219, 200, 88, 176, 158, 63, 253, 127] = sha256("global:update")[:8]
    rq.data = bytearray().join([
        bytearray([219, 200, 88, 176, 158, 63, 253, 127]),  # update 方法的 discriminator
        len(args.args[1].encode()).to_bytes(4, 'little'),   # 数据长度（4字节小端序）
        args.args[1].encode(),                              # 实际数据内容
    ])
    
    # 构造、签名、发送交易
    tx = pxsol.core.Transaction.requisition_decode(user.pubkey, [rq])
    tx.message.recent_blockhash = pxsol.base58.decode(pxsol.rpc.get_latest_blockhash({})['blockhash'])
    tx.sign([user.prikey])
    txid = pxsol.rpc.send_transaction(base64.b64encode(tx.serialize()).decode(), {})
    
    # 等待交易确认并打印日志
    pxsol.rpc.wait([txid])
    r = pxsol.rpc.get_transaction(txid, {})
    for e in r['meta']['logMessages']:
        print(e)


def load():
    """读取账户中存储的数据"""
    # 查询链上账户信息
    info = pxsol.rpc.get_account_info(data_pubkey.base58(), {})
    
    # 解码并跳过账户头部，只打印实际数据
    # 账户布局：8 (discriminator) + 32 (auth) + 1 (bump) + 4 (Vec长度) + 数据
    print(base64.b64decode(info['data'][0])[8 + 32 + 1 + 4:].decode())


if __name__ == '__main__':
    # 根据命令行参数调用对应函数
    # 用法：python pxsol-ss-anchor.py init
    #      python pxsol-ss-anchor.py update "Hello"
    #      python pxsol-ss-anchor.py load
    eval(f'{args.args[0]}()')
