.PHONY: help build build-debug test test-sbf clean deploy logs

# 默认目标
help:
	@echo "Pinocchio Vault - 可用命令:"
	@echo ""
	@echo "  make build        - 构建程序 (生产版本,启用性能优化)"
	@echo "  make build-debug  - 构建程序 (调试版本,包含日志)"
	@echo "  make test         - 运行单元测试"
	@echo "  make test-sbf     - 运行 Solana 程序测试"
	@echo "  make clean        - 清理构建文件"
	@echo "  make deploy       - 部署到本地测试网"
	@echo "  make logs         - 查看程序日志"
	@echo "  make validator    - 启动本地测试验证器"
	@echo ""

# 构建生产版本
build:
	@echo "构建生产版本 (启用性能优化)..."
	cargo build-sbf --release || cargo build-bpf --release

# 构建调试版本
build-debug:
	@echo "构建调试版本 (包含日志)..."
	cargo build-sbf --no-default-features || cargo build-bpf --no-default-features

# 运行单元测试
test:
	@echo "运行单元测试..."
	cargo test

# 运行 Solana 程序测试
test-sbf:
	@echo "运行 Solana 程序测试..."
	cargo test-sbf || cargo test-bpf

# 清理构建文件
clean:
	@echo "清理构建文件..."
	cargo clean
	rm -rf target/
	rm -rf test-ledger/

# 部署到本地测试网
deploy:
	@echo "部署程序到本地测试网..."
	@if [ ! -f target/deploy/vault_program.so ]; then \
		echo "错误: 程序文件不存在,请先运行 'make build'"; \
		exit 1; \
	fi
	solana program deploy target/deploy/vault_program.so

# 查看程序日志
logs:
	@echo "查看程序日志 (Ctrl+C 退出)..."
	solana logs

# 启动本地测试验证器
validator:
	@echo "启动本地测试验证器..."
	solana-test-validator

# 检查代码格式
fmt:
	@echo "格式化代码..."
	cargo fmt

# 代码检查
clippy:
	@echo "运行 Clippy 检查..."
	cargo clippy -- -D warnings

# 完整检查 (格式化 + Clippy + 测试)
check: fmt clippy test
	@echo "✓ 所有检查通过"
