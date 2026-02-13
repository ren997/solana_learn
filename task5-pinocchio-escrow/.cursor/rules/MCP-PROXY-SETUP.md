# Cursor MCP 代理配置指南

## 问题说明

如果 MCP 工具无法访问（如 Solana MCP 服务器），通常是因为网络连接问题，需要配置代理。

## 配置方法

### 方法一：通过环境变量配置（推荐）

#### 1. 在 WSL2 中配置代理

如果你使用的是 WSL2，需要单独配置代理：

```bash
# 获取 Windows 主机的 IP 地址
export hostip=$(cat /etc/resolv.conf | grep nameserver | awk '{print $2}')

# 设置代理环境变量（根据你的代理软件端口调整）
export http_proxy="http://${hostip}:7897"
export https_proxy="http://${hostip}:7897"
export all_proxy="http://${hostip}:7897"
export HTTP_PROXY="http://${hostip}:7897"
export HTTPS_PROXY="http://${hostip}:7897"
export ALL_PROXY="http://${hostip}:7897"

# 永久配置：添加到 ~/.bashrc 或 ~/.zshrc
cat >> ~/.bashrc << 'EOF'

# 代理设置
export hostip=$(cat /etc/resolv.conf | grep nameserver | awk '{print $2}')
export http_proxy="http://${hostip}:7897"
export https_proxy="http://${hostip}:7897"
export all_proxy="http://${hostip}:7897"
export HTTP_PROXY="http://${hostip}:7897"
export HTTPS_PROXY="http://${hostip}:7897"
export ALL_PROXY="http://${hostip}:7897"
EOF

# 使配置生效
source ~/.bashrc
```

**重要提示：**
- `7897` 是 Clash 的默认端口，如果你的代理端口不同，请修改
- 常见代理端口：
  - Clash: `7897` (HTTP) 或 `7890` (SOCKS5)
  - V2Ray: `10808` (SOCKS5) 或 `10809` (HTTP)
  - Shadowsocks: `1080` (SOCKS5)

#### 2. 验证代理是否生效

```bash
# 测试 HTTP 连接
curl -I https://google.com

# 测试 HTTPS 连接
curl -I https://solana.com
```

### 方法二：在 Cursor 启动脚本中配置

创建或编辑 Cursor 的启动脚本，添加代理环境变量：

```bash
# 创建启动脚本
cat > ~/start-cursor-with-proxy.sh << 'EOF'
#!/bin/bash
export hostip=$(cat /etc/resolv.conf | grep nameserver | awk '{print $2}')
export http_proxy="http://${hostip}:7897"
export https_proxy="http://${hostip}:7897"
export all_proxy="http://${hostip}:7897"
export HTTP_PROXY="http://${hostip}:7897"
export HTTPS_PROXY="http://${hostip}:7897"
export ALL_PROXY="http://${hostip}:7897"

# 启动 Cursor（根据你的安装路径调整）
/usr/bin/cursor "$@"
EOF

chmod +x ~/start-cursor-with-proxy.sh
```

### 方法三：配置系统级代理（适用于 Linux 桌面环境）

#### 使用 systemd 环境变量

```bash
# 创建 systemd 用户环境配置
mkdir -p ~/.config/environment.d
cat > ~/.config/environment.d/proxy.conf << 'EOF'
http_proxy=http://127.0.0.1:7897
https_proxy=http://127.0.0.1:7897
all_proxy=http://127.0.0.1:7897
HTTP_PROXY=http://127.0.0.1:7897
HTTPS_PROXY=http://127.0.0.1:7897
ALL_PROXY=http://127.0.0.1:7897
EOF
```

然后重启 Cursor。

### 方法四：通过 Cursor 设置界面配置

1. 打开 Cursor
2. 进入设置：`File` > `Preferences` > `Settings`
3. 搜索 "proxy"
4. 配置以下设置：
   - `http.proxy`: `http://127.0.0.1:7897`
   - `http.proxyStrictSSL`: `false` (如果使用自签名证书)

## 常见代理软件配置

### Clash for Windows

1. 打开 Clash
2. 进入 `Settings` > `General`
3. 确保 `Allow LAN` 已启用
4. 查看端口设置（默认 HTTP: 7897, SOCKS5: 7890）

### V2Ray

1. 查看 V2Ray 配置中的 `inbounds` 端口
2. 确保 `allowLan: true` 已设置
3. 使用对应的端口配置代理

## 故障排查

### 1. 检查代理是否运行

```bash
# 检查代理端口是否开放
netstat -tuln | grep 7897

# 或使用 ss 命令
ss -tuln | grep 7897
```

### 2. 检查防火墙设置

确保 Windows 防火墙允许代理软件接受局域网连接。

### 3. 测试代理连接

```bash
# 测试代理是否工作
curl -x http://127.0.0.1:7897 https://www.google.com

# 或使用环境变量
http_proxy=http://127.0.0.1:7897 curl https://www.google.com
```

### 4. 检查 MCP 服务器连接

```bash
# 测试 Solana MCP 服务器连接
curl -I https://api.solana.com

# 如果使用代理
http_proxy=http://127.0.0.1:7897 curl -I https://api.solana.com
```

### 5. 查看 Cursor 日志

检查 Cursor 的错误日志：
- Linux: `~/.config/Cursor/logs/`
- 查看是否有网络连接错误

## 针对不同代理类型的配置

### HTTP 代理

```bash
export http_proxy="http://127.0.0.1:7897"
export https_proxy="http://127.0.0.1:7897"
```

### SOCKS5 代理

```bash
export http_proxy="socks5://127.0.0.1:7890"
export https_proxy="socks5://127.0.0.1:7890"
```

### 带认证的代理

```bash
export http_proxy="http://username:password@127.0.0.1:7897"
export https_proxy="http://username:password@127.0.0.1:7897"
```

## 验证 MCP 是否正常工作

配置完成后，重启 Cursor，然后测试 MCP 工具：

1. 在 Cursor 中打开聊天
2. 尝试使用 Solana MCP 工具
3. 如果仍然无法连接，检查：
   - 代理是否正确配置
   - 代理软件是否运行
   - 防火墙是否阻止连接

## 注意事项

1. **WSL2 特殊处理**：WSL2 需要单独配置代理，不能直接使用 Windows 的代理设置
2. **端口号**：根据你的代理软件实际端口修改
3. **重启生效**：修改环境变量后需要重启 Cursor
4. **代理软件设置**：确保代理软件开启了"允许局域网连接"
