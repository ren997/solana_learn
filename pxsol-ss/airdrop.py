import pxsol

# 配置连接到本地验证器
pxsol.config.current.rpc_url = "http://localhost:8899"

# 创建用户钱包（使用固定私钥，方便测试）
user = pxsol.wallet.Wallet(pxsol.core.PriKey.int_decode(0x02))

print(f"用户地址: {user.pubkey.base58()}")

# 检查余额
balance = pxsol.rpc.get_balance(user.pubkey.base58(), {})
print(f"当前余额: {balance} lamports ({balance/1e9} SOL)")

# 如果余额不足，进行 airdrop
if balance < 100000000:  # 少于 0.1 SOL
    print("余额不足，正在请求 airdrop...")
    txid = pxsol.rpc.request_airdrop(user.pubkey.base58(), 1000000000, {})  # 1 SOL
    print(f"Airdrop 交易 ID: {txid}")
    
    # 等待交易确认
    pxsol.rpc.wait([txid])
    
    # 再次检查余额
    balance = pxsol.rpc.get_balance(user.pubkey.base58(), {})
    print(f"新余额: {balance} lamports ({balance/1e9} SOL)")
else:
    print("余额充足，无需 airdrop")
