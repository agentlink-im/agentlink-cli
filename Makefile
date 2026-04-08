# AgentLink CLI Makefile
# 提供便捷的构建和发布命令

.PHONY: help build release test clean install lint fmt check

# 默认目标
.DEFAULT_GOAL := help

# 变量
BINARY_NAME := agentlink
VERSION := $(shell grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
RELEASE_DIR := release/v$(VERSION)

# 颜色定义
BLUE := \033[36m
GREEN := \033[32m
YELLOW := \033[33m
RED := \033[31m
NC := \033[0m

help: ## 显示帮助信息
	@echo "$(BLUE)AgentLink CLI Makefile$(NC)"
	@echo ""
	@echo "$(GREEN)可用命令:$(NC)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(YELLOW)%-15s$(NC) %s\n", $$1, $$2}'

build: ## 构建发布版本
	@echo "$(BLUE)构建 $(BINARY_NAME) v$(VERSION)...$(NC)"
	@cargo build --release
	@echo "$(GREEN)构建完成: target/release/$(BINARY_NAME)$(NC)"

build-all: ## 构建所有支持的平台（需要交叉编译工具链）
	@echo "$(BLUE)构建所有平台...$(NC)"
	@./scripts/build-release.sh v$(VERSION)

test: ## 运行测试
	@echo "$(BLUE)运行测试...$(NC)"
	@cargo test

lint: ## 运行代码检查
	@echo "$(BLUE)运行 Clippy...$(NC)"
	@cargo clippy -- -D warnings

fmt: ## 格式化代码
	@echo "$(BLUE)格式化代码...$(NC)"
	@cargo fmt

check: ## 检查代码（不编译）
	@echo "$(BLUE)检查代码...$(NC)"
	@cargo check

install: build ## 安装到系统 (需要 sudo 权限)
	@echo "$(BLUE)安装 $(BINARY_NAME) 到 /usr/local/bin...$(NC)"
	@cp target/release/$(BINARY_NAME) /usr/local/bin/
	@chmod +x /usr/local/bin/$(BINARY_NAME)
	@echo "$(GREEN)安装完成!$(NC)"

install-local: build ## 安装到 ~/.cargo/bin
	@echo "$(BLUE)安装 $(BINARY_NAME) 到 ~/.cargo/bin...$(NC)"
	@mkdir -p ~/.cargo/bin
	@cp target/release/$(BINARY_NAME) ~/.cargo/bin/
	@chmod +x ~/.cargo/bin/$(BINARY_NAME)
	@echo "$(GREEN)安装完成!$(NC)"
	@echo "$(YELLOW)请确保 ~/.cargo/bin 在你的 PATH 中$(NC)"

uninstall: ## 从系统卸载
	@echo "$(BLUE)卸载 $(BINARY_NAME)...$(NC)"
	@rm -f /usr/local/bin/$(BINARY_NAME)
	@rm -f ~/.cargo/bin/$(BINARY_NAME)
	@echo "$(GREEN)卸载完成!$(NC)"

release: build ## 创建 GitHub Release 并上传二进制文件
	@echo "$(BLUE)创建 Release v$(VERSION)...$(NC)"
	@./scripts/build-release.sh v$(VERSION)

release-draft: build ## 创建草稿 Release（不自动发布）
	@echo "$(BLUE)创建草稿 Release v$(VERSION)...$(NC)"
	@./scripts/build-release.sh v$(VERSION) --draft

clean: ## 清理构建文件
	@echo "$(BLUE)清理构建文件...$(NC)"
	@cargo clean
	@rm -rf release/
	@echo "$(GREEN)清理完成!$(NC)"

run: ## 运行开发版本
	@cargo run --

dev: ## 运行开发版本（带参数）
	@cargo run -- $(ARGS)

update: ## 更新依赖
	@echo "$(BLUE)更新依赖...$(NC)"
	@cargo update

version: ## 显示当前版本
	@echo "$(GREEN)$(BINARY_NAME) v$(VERSION)$(NC)"

# Docker 命令
docker-build: ## 构建 Docker 镜像
	@echo "$(BLUE)构建 Docker 镜像...$(NC)"
	@docker build -t agentlink/cli:latest -t agentlink/cli:v$(VERSION) .
	@echo "$(GREEN)Docker 镜像构建完成!$(NC)"

docker-run: ## 运行 Docker 容器（交互式）
	@echo "$(BLUE)启动 Docker 容器...$(NC)"
	@docker run -it --rm -v $(HOME)/.config/agentlink:/home/agentlink/.config/agentlink agentlink/cli:latest

docker-push: ## 推送 Docker 镜像到仓库
	@echo "$(BLUE)推送 Docker 镜像...$(NC)"
	@docker push agentlink/cli:latest
	@docker push agentlink/cli:v$(VERSION)
	@echo "$(GREEN)Docker 镜像推送完成!$(NC)"

# 代码签名
sign: ## 对发布二进制进行代码签名（需要设置签名环境变量）
	@echo "$(BLUE)代码签名...$(NC)"
	@./scripts/sign-release.sh $(RELEASE_DIR)
	@echo "$(GREEN)代码签名完成!$(NC)"

# Homebrew 更新
homebrew-update: ## 更新 Homebrew 公式（默认同步到 ../homebrew-tap）
	@echo "$(BLUE)更新 Homebrew 公式...$(NC)"
	@./scripts/update-homebrew.sh v$(VERSION) ../homebrew-tap $(RELEASE_DIR)
	@echo "$(GREEN)Homebrew 公式更新完成!$(NC)"

homebrew-update-local: ## 仅更新本地公式副本
	@echo "$(BLUE)更新本地 Homebrew 公式...$(NC)"
	@./scripts/update-homebrew.sh v$(VERSION) ./homebrew-tap $(RELEASE_DIR)
	@echo "$(GREEN)本地 Homebrew 公式更新完成!$(NC)"

homebrew-trigger: ## 触发 GitHub Actions 自动更新 Homebrew 公式
	@echo "$(BLUE)触发 Homebrew Tap 自动更新...$(NC)"
	@gh api repos/agentlink-im/homebrew-tap/dispatches \
		--input - <<< '{"event_type": "release-published", "client_payload": {"version": "v$(VERSION)"}}' \
		&& echo "$(GREEN)自动更新已触发!$(NC)" \
		|| echo "$(RED)触发失败，请检查权限$(NC)"

# 快速发布流程
ship: fmt lint test release ## 完整发布流程：格式化 -> 检查 -> 测试 -> 发布
	@echo "$(GREEN)发布流程完成!$(NC)"

ship-all: fmt lint test release sign homebrew-update docker-build ## 完整发布流程（包含签名、Homebrew、Docker）
	@echo "$(GREEN)完整发布流程完成!$(NC)"
