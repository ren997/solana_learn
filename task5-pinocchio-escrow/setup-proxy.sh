#!/bin/bash

# Cursor MCP 代理配置脚本
# 使用方法: bash setup-proxy.sh [代理端口]

# 默认代理端口（Clash 默认是 7897）
PROXY_PORT=${1:-7897}

echo "=========================================="
echo "Cursor MCP 代理配置脚本"
echo "=========================================="
echo ""

# 检测是否在 WSL2 环境
if [ -f /proc/version ] && grep -q Microsoft /proc/version; then
    echo "检测到 WSL2 环境"
    # 获取 Windows 主机 IP
    HOST_IP=$(cat /etc/resolv.conf | grep nameserver | awk '{print $2}' | head -1)
    PROXY_URL="http://${HOST_IP}:${PROXY_PORT}"
    echo "Windows 主机 IP: $HOST_IP"
else
    echo "检测到 Linux 环境"
    PROXY_URL="http://127.0.0.1:${PROXY_PORT}"
fi

echo "代理地址: $PROXY_URL"
echo ""

# 询问用户确认
read -p "是否使用此代理配置? (y/n): " confirm
if [ "$confirm" != "y" ] && [ "$confirm" != "Y" ]; then
    echo "已取消配置"
    exit 0
fi

# 配置代理环境变量
echo ""
echo "正在配置代理环境变量..."

# 检测使用的 shell
if [ -n "$ZSH_VERSION" ]; then
    SHELL_RC="$HOME/.zshrc"
elif [ -n "$BASH_VERSION" ]; then
    SHELL_RC="$HOME/.bashrc"
else
    SHELL_RC="$HOME/.bashrc"
fi

# 备份原配置文件
if [ -f "$SHELL_RC" ]; then
    cp "$SHELL_RC" "${SHELL_RC}.backup.$(date +%Y%m%d_%H%M%S)"
    echo "已备份配置文件: ${SHELL_RC}.backup.*"
fi

# 移除旧的代理配置（如果存在）
sed -i '/# Cursor MCP 代理配置/,/# 代理配置结束/d' "$SHELL_RC"

# 添加新的代理配置
cat >> "$SHELL_RC" << EOF

# Cursor MCP 代理配置
if [ -f /proc/version ] && grep -q Microsoft /proc/version; then
    # WSL2 环境
    export hostip=\$(cat /etc/resolv.conf | grep nameserver | awk '{print \$2}' | head -1)
    export http_proxy="http://\${hostip}:${PROXY_PORT}"
    export https_proxy="http://\${hostip}:${PROXY_PORT}"
    export all_proxy="http://\${hostip}:${PROXY_PORT}"
else
    # Linux 环境
    export http_proxy="${PROXY_URL}"
    export https_proxy="${PROXY_URL}"
    export all_proxy="${PROXY_URL}"
fi
export HTTP_PROXY="\$http_proxy"
export HTTPS_PROXY="\$https_proxy"
export ALL_PROXY="\$all_proxy"
# 代理配置结束
EOF

echo "代理配置已添加到: $SHELL_RC"
echo ""

# 使配置立即生效
echo "正在使配置生效..."
source "$SHELL_RC" 2>/dev/null || true

# 显示当前配置
echo ""
echo "=========================================="
echo "当前代理配置:"
echo "=========================================="
echo "http_proxy:  $http_proxy"
echo "https_proxy: $https_proxy"
echo "HTTP_PROXY:  $HTTP_PROXY"
echo "HTTPS_PROXY: $HTTPS_PROXY"
echo ""

# 测试代理连接
echo "正在测试代理连接..."
if curl -s --proxy "$http_proxy" -I https://www.google.com > /dev/null 2>&1; then
    echo "✅ 代理连接测试成功！"
else
    echo "❌ 代理连接测试失败！"
    echo "请检查："
    echo "  1. 代理软件是否运行"
    echo "  2. 代理端口是否正确（当前: $PROXY_PORT）"
    echo "  3. 代理软件是否开启了'允许局域网连接'"
fi

echo ""
echo "=========================================="
echo "配置完成！"
echo "=========================================="
echo ""
echo "重要提示："
echo "1. 请重启 Cursor 编辑器使配置生效"
echo "2. 如果使用新终端，运行: source $SHELL_RC"
echo "3. 如果代理端口不同，请修改脚本中的 PROXY_PORT 或重新运行:"
echo "   bash setup-proxy.sh [你的端口号]"
echo ""
