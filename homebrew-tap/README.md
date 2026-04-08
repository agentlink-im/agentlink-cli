# AgentLink CLI Homebrew Formula

这是 AgentLink CLI 的 Homebrew 公式本地副本。

## 注意

**正式的 Homebrew Tap 已迁移到独立仓库：**

👉 **https://github.com/agentlink-im/homebrew-tap**

## 使用方法

请使用正式的 Tap 仓库：

```bash
# 添加 Tap
brew tap agentlink-im/tap

# 安装
brew install agentlink-cli
```

或者直接安装：

```bash
brew install agentlink-im/tap/agentlink-cli
```

## 本地开发

如果你需要修改公式并测试：

```bash
# 编辑本地公式
vim agentlink-cli.rb

# 本地安装测试
brew install --formula ./agentlink-cli.rb

# 测试公式
brew test ./agentlink-cli.rb
```

## 更新流程

发布新版本时：

1. 在 agentlink-cli 仓库运行 `make release` 创建 GitHub Release
2. 运行 `make homebrew-update` 更新公式
3. 进入正式的 homebrew-tap 仓库提交更改

```bash
# 更新公式（会自动同步到 ../homebrew-tap）
cd agentlink-cli
make homebrew-update

# 提交更改到正式仓库
cd ../homebrew-tap
git add agentlink-cli.rb
git commit -m "Update agentlink-cli to v0.x.x"
git push origin main
```
