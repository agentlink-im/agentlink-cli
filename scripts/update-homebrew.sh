#!/bin/bash
# 更新 Homebrew Tap 公式脚本
# 使用方法: ./scripts/update-homebrew.sh <version> [tap_repo_path]
#
# 示例:
#   ./scripts/update-homebrew.sh v0.1.0 ../homebrew-tap
#   ./scripts/update-homebrew.sh v0.1.0 /path/to/homebrew-tap

set -e

VERSION="${1#v}"  # 去掉 v 前缀
TAP_REPO_PATH="${2:-../homebrew-tap}"
RELEASE_DIR="${3:-release/v${VERSION}}"
FORMULA_FILE="${TAP_REPO_PATH}/agentlink-cli.rb"

if [ -z "$VERSION" ]; then
    echo "错误: 请提供版本号"
    echo "用法: $0 <version> [tap_repo_path] [release_dir]"
    echo ""
    echo "示例:"
    echo "  $0 v0.1.0                          # 使用默认路径 ../homebrew-tap"
    echo "  $0 v0.1.0 /path/to/homebrew-tap    # 指定 tap 仓库路径"
    exit 1
fi

if [ ! -d "$TAP_REPO_PATH" ]; then
    echo "错误: Homebrew Tap 仓库目录不存在: $TAP_REPO_PATH"
    echo ""
    echo "请克隆仓库:"
    echo "  git clone https://github.com/agentlink-im/homebrew-tap.git ../homebrew-tap"
    exit 1
fi

if [ ! -f "$FORMULA_FILE" ]; then
    echo "错误: 公式文件不存在: $FORMULA_FILE"
    exit 1
fi

if [ ! -f "${RELEASE_DIR}/checksums.txt" ]; then
    echo "错误: checksums.txt 不存在: ${RELEASE_DIR}/checksums.txt"
    exit 1
fi

echo "========================================"
echo "  更新 Homebrew Tap 公式"
echo "========================================"
echo ""
echo "版本: $VERSION"
echo "Tap 仓库: $TAP_REPO_PATH"
echo "发布目录: $RELEASE_DIR"
echo ""

# 读取 checksums.txt
declare -A checksums
while read -r sum file; do
    checksums[$file]=$sum
done < "${RELEASE_DIR}/checksums.txt"

echo "发现以下校验和:"
for key in "${!checksums[@]}"; do
    echo "  $key: ${checksums[$key]}"
done
echo ""

# 更新公式
sed -i.bak \
    -e "s/version \"[0-9.]*\"/version \"${VERSION}\"/" \
    -e "s/PLACEHOLDER_SHA256_MACOS_AARCH64/${checksums[agentlink-macos-aarch64]:-PLACEHOLDER_SHA256_MACOS_AARCH64}/" \
    -e "s/PLACEHOLDER_SHA256_MACOS_X86_64/${checksums[agentlink-macos-x86_64]:-PLACEHOLDER_SHA256_MACOS_X86_64}/" \
    -e "s/PLACEHOLDER_SHA256_LINUX_AARCH64/${checksums[agentlink-linux-aarch64]:-PLACEHOLDER_SHA256_LINUX_AARCH64}/" \
    -e "s/PLACEHOLDER_SHA256_LINUX_X86_64/${checksums[agentlink-linux-x86_64]:-PLACEHOLDER_SHA256_LINUX_X86_64}/" \
    "$FORMULA_FILE"

rm -f "${FORMULA_FILE}.bak"

echo "✅ Homebrew 公式已更新: $FORMULA_FILE"
echo ""

# 显示更新后的内容
echo "更新后的公式内容:"
echo "========================================"
cat "$FORMULA_FILE"
echo ""
echo "========================================"
echo ""

# 提示提交更改
echo "下一步操作:"
echo ""
echo "  cd $TAP_REPO_PATH"
echo "  git add agentlink-cli.rb"
echo "  git commit -m \"Update agentlink-cli to v${VERSION}\""
echo "  git push origin main"
echo ""
echo "或者使用快捷命令:"
echo "  (cd $TAP_REPO_PATH && git add . && git commit -m \"Update agentlink-cli to v${VERSION}\" && git push)"
