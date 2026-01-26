import argparse
import base64
import json
import pxsol
import random
import subprocess

parser = argparse.ArgumentParser()
parser.add_argument('--net', type=str, choices=['develop', 'mainnet', 'testnet'], default='develop')
parser.add_argument('--prikey', type=str, default='11111111111111111111111111111112')
parser.add_argument('args', nargs='+')
args = parser.parse_args()

if args.net == 'develop':
    pxsol.config.current = pxsol.config.develop
if args.net == 'mainnet':
    pxsol.config.current = pxsol.config.mainnet
if args.net == 'testnet':
    pxsol.config.current = pxsol.config.testnet
pxsol.config.current.log = 1


def call(c: str):
    return subprocess.run(c, check=True, shell=True)


def info_save(k: str, v: str) -> None:
    with open('res/info.json', 'r') as f:
        info = json.load(f)
    info[args.net][k] = v
    with open('res/info.json', 'w') as f:
        json.dump(info, f, indent=4)


def info_load(k: str) -> str:
    with open('res/info.json', 'r') as f:
        info = json.load(f)
    return info[args.net][k]


def deploy():
    # Create spl mint
    user = pxsol.wallet.Wallet(pxsol.core.PriKey.base58_decode(args.prikey))
    pxsol.log.debugln(f'main: create mint')
    pubkey_mint = user.spl_create(9, {
        'metadata': {
            'name': 'PXSOL',
            'symbol': 'PXS',
            'uri': 'https://raw.githubusercontent.com/mohanson/pxsol/refs/heads/master/res/pxs.json',
        }
    })
    pxsol.log.debugln(f'main: create mint pubkey={pubkey_mint}')
    info_save('pubkey_mint', pubkey_mint.base58())

    # Mint spl tokens
    pxsol.log.debugln(f'main: mint 100000000 for {user.pubkey}')
    user.spl_mint(pubkey_mint, user.pubkey, 100000000 * 10**9)

    # Deploy spl mana
    call('cargo build-sbf -- -Znext-lockfile-bump')
    pxsol.log.debugln(f'main: deploy mana')
    with open('target/deploy/pxsol_spl.so', 'rb') as f:
        data = bytearray(f.read())
    pubkey_mana = user.program_deploy(data)
    pxsol.log.debugln(f'main: deploy mana pubkey={pubkey_mana}')
    info_save('pubkey_mana', pubkey_mana.base58())

    # Send spl tokens
    pubkey_mana_seed = bytearray([])
    pubkey_mana_auth = pubkey_mana.derive_pda(pubkey_mana_seed)[0]
    user.spl_transfer(pubkey_mint, pubkey_mana_auth, 100000000 * 10**9)


def update():
    # Update spl mana
    user = pxsol.wallet.Wallet(pxsol.core.PriKey.base58_decode(args.prikey))
    pubkey_mana = pxsol.core.PubKey.base58_decode(info_load('pubkey_mana'))
    call('cargo build-sbf -- -Znext-lockfile-bump')
    pxsol.log.debugln(f'main: update mana')
    with open('target/deploy/pxsol_spl.so', 'rb') as f:
        user.program_update(pubkey_mana, bytearray(f.read()))


def genuser():
    pxsol.log.debugln(f'main: random user')
    user = pxsol.wallet.Wallet(pxsol.core.PriKey(bytearray(random.randbytes(32))))
    pxsol.log.debugln(f'main: random user prikey={user.prikey}')
    pxsol.log.debugln(f'main: random user pubkey={user.pubkey}')
    pxsol.log.debugln(f'main: request sol airdrop')
    txid = pxsol.rpc.request_airdrop(user.pubkey.base58(), 1 * pxsol.denomination.sol, {})
    pxsol.log.debugln(f'main: request sol airdrop txid={txid}')
    pxsol.rpc.wait([txid])
    pxsol.log.debugln(f'main: request sol airdrop done')


def airdrop():
    # 1. 准备账户地址
    user = pxsol.wallet.Wallet(pxsol.core.PriKey.base58_decode(args.prikey))  # 用户钱包（领取空投的人）
    pubkey_mint = pxsol.core.PubKey.base58_decode(info_load('pubkey_mint'))  # 代币 Mint 地址
    pubkey_mana = pxsol.core.PubKey.base58_decode(info_load('pubkey_mana'))  # 空投程序地址
    
    # 2. 派生程序的 PDA 和其代币账户
    pubkey_mana_seed = bytearray([])  # 空种子
    pubkey_mana_auth = pubkey_mana.derive_pda(pubkey_mana_seed)[0]  # 程序的 PDA（持有代币）
    pubkey_mana_spla = pxsol.wallet.Wallet.view_only(pubkey_mana_auth).spl_account(pubkey_mint)  # PDA 的代币账户（ATA）
    
    # 3. 构建交易指令（Instruction）
    rq = pxsol.core.Requisition(pubkey_mana, [], bytearray())  # 创建指令，调用空投程序
    # 添加指令需要的账户（按顺序传递给程序）
    rq.account.append(pxsol.core.AccountMeta(user.pubkey, 3))  # [0] 用户账户（签名者 + 可写）
    rq.account.append(pxsol.core.AccountMeta(user.spl_account(pubkey_mint), 1))  # [1] 用户的代币账户（可写）
    rq.account.append(pxsol.core.AccountMeta(pubkey_mana, 0))  # [2] 程序账户（只读）
    rq.account.append(pxsol.core.AccountMeta(pubkey_mana_auth, 0))  # [3] 程序 PDA（只读）
    rq.account.append(pxsol.core.AccountMeta(pubkey_mana_spla, 1))  # [4] PDA 的代币账户（可写）
    rq.account.append(pxsol.core.AccountMeta(pubkey_mint, 0))  # [5] 代币 Mint（只读）
    rq.account.append(pxsol.core.AccountMeta(pxsol.program.System.pubkey, 0))  # [6] 系统程序
    rq.account.append(pxsol.core.AccountMeta(pxsol.program.Token.pubkey, 0))  # [7] Token 程序
    rq.account.append(pxsol.core.AccountMeta(pxsol.program.AssociatedTokenAccount.pubkey, 0))  # [8] ATA 程序
    rq.data = bytearray()  # 指令数据（空，程序不需要额外参数）
    
    # 4. 构建交易
    tx = pxsol.core.Transaction.requisition_decode(user.pubkey, [rq])  # 将指令打包成交易
    tx.message.recent_blockhash = pxsol.base58.decode(pxsol.rpc.get_latest_blockhash({})['blockhash'])  # 设置最新区块哈希
    tx.sign([user.prikey])  # 用用户私钥签名
    
    # 5. 发送交易并等待确认
    pxsol.log.debugln(f'main: request spl airdrop')
    txid = pxsol.rpc.send_transaction(base64.b64encode(tx.serialize()).decode(), {})  # 发送交易
    pxsol.rpc.wait([txid])  # 等待交易确认
    
    # 6. 查询交易日志和余额
    tlog = pxsol.rpc.get_transaction(txid, {})  # 获取交易详情
    for e in tlog['meta']['logMessages']:
        pxsol.log.debugln(e)  # 打印程序日志
    splcnt = user.spl_balance(pubkey_mint)  # 查询领取后的代币余额
    pxsol.log.debugln(f'main: request spl airdrop done recv={splcnt[0] / 10**splcnt[1]}')


if __name__ == '__main__':
    eval(f'{args.args[0]}()')
