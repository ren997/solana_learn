# 如何将 Solana 规则配置为全局规则

## 方法一：使用命令行（推荐）

在终端中执行以下命令：

```bash
# 创建全局规则目录（如果不存在）
mkdir -p ~/.cursor/rules

# 复制规则文件到全局目录
cp /home/dmin/workspace/solana_learn/task5-pinocchio-escrow/.cursor/rules/solana-global.mdc ~/.cursor/rules/

# 或者如果 ~/.cursor/rules 不存在，尝试 ~/.config/Cursor/rules
mkdir -p ~/.config/Cursor/rules
cp /home/dmin/workspace/solana_learn/task5-pinocchio-escrow/.cursor/rules/solana-global.mdc ~/.config/Cursor/rules/
```

## 方法二：手动操作

1. 打开文件管理器
2. 导航到：`/home/dmin/workspace/solana_learn/task5-pinocchio-escrow/.cursor/rules/`
3. 复制 `solana-global.mdc` 文件
4. 粘贴到以下位置之一：
   - `~/.cursor/rules/` （用户主目录下的 .cursor/rules 文件夹）
   - `~/.config/Cursor/rules/` （Cursor 配置目录）

## 验证配置

配置完成后，重启 Cursor 编辑器，规则将自动应用到所有项目。

## 规则作用范围

- **项目级别规则** (`项目/.cursor/rules/`): 只适用于当前项目
- **全局规则** (`~/.cursor/rules/` 或 `~/.config/Cursor/rules/`): 适用于所有项目

## 注意事项

- 全局规则会应用到所有项目，包括非 Solana 项目
- 如果某个项目有同名的规则文件，项目级别的规则会优先
- 可以同时存在多个规则文件，它们会合并应用
