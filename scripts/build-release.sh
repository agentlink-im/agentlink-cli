#!/bin/bash
# AgentLink CLI 本地构建发布脚本
# 使用方法: ./scripts/build-release.sh [version]
# 示例: ./scripts/build-release.sh v0.1.0

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# 获取版本号
if [ -z "$1" ]; then
    # 从 Cargo.toml 读取版本
    VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
    VERSION="v${VERSION}"
    print_info "未指定版本，使用 Cargo.toml 版本: $VERSION"
else
    VERSION="$1"
    # 确保版本号以 v 开头
    if [[ ! "$VERSION" =~ ^v ]]; then
        VERSION="v${VERSION}"
    fi
fi

print_info "构建版本: $VERSION"

# 创建发布目录
RELEASE_DIR="release/${VERSION}"
mkdir -p "$RELEASE_DIR"

# 检查依赖
check_dependencies() {
    print_info "检查依赖..."
    
    if ! command -v cargo &> /dev/null; then
        print_error "需要安装 Rust/Cargo"
        exit 1
    fi
    
    if ! command -v gh &> /dev/null; then
        print_warning "未安装 GitHub CLI (gh)，将无法自动创建 Release"
        print_info "安装指南: https://cli.github.com/"
    fi
    
    print_success "依赖检查完成"
}

# 本地平台构建
build_local() {
    local target
    target=$(rustc -vV | grep host | cut -d' ' -f2)
    
    print_info "构建本地平台: $target"
    
    cargo build --release
    
    local os
    local arch
    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    arch=$(uname -m)
    
    case "$os" in
        linux)
            case "$arch" in
                x86_64)     asset_name="agentlink-linux-x86_64";;
                aarch64)    asset_name="agentlink-linux-aarch64";;
                armv7l)     asset_name="agentlink-linux-armv7";;
                *)          asset_name="agentlink-linux-${arch}";;
            esac
            ;;
        darwin)
            case "$arch" in
                x86_64)     asset_name="agentlink-macos-x86_64";;
                arm64)      asset_name="agentlink-macos-aarch64";;
                *)          asset_name="agentlink-macos-${arch}";;
            esac
            ;;
        *)
            asset_name="agentlink-${os}-${arch}";;
    esac
    
    cp "target/release/agentlink" "${RELEASE_DIR}/${asset_name}"
    print_success "本地构建完成: ${asset_name}"
}

# 交叉编译其他平台
cross_compile() {
    print_info "开始交叉编译..."
    
    # 定义目标平台
    local targets=(
        "x86_64-unknown-linux-musl"
        "aarch64-unknown-linux-gnu"
        "x86_64-apple-darwin"
        "aarch64-apple-darwin"
    )
    
    for target in "${targets[@]}"; do
        print_info "编译目标: $target"
        
        # 检查是否支持该目标
        if ! rustup target list --installed | grep -q "$target"; then
            print_warning "目标 $target 未安装，尝试安装..."
            if ! rustup target add "$target" 2>/dev/null; then
                print_warning "无法安装目标 $target，跳过"
                continue
            fi
        fi
        
        # 构建
        if cargo build --release --target "$target" 2>/dev/null; then
            # 确定输出文件名
            local asset_name=""
            case "$target" in
                x86_64-unknown-linux-musl)  asset_name="agentlink-linux-x86_64-musl";;
                aarch64-unknown-linux-gnu)  asset_name="agentlink-linux-aarch64";;
                x86_64-apple-darwin)        asset_name="agentlink-macos-x86_64";;
                aarch64-apple-darwin)       asset_name="agentlink-macos-aarch64";;
            esac
            
            if [ -n "$asset_name" ]; then
                cp "target/${target}/release/agentlink" "${RELEASE_DIR}/${asset_name}"
                print_success "构建完成: ${asset_name}"
            fi
        else
            print_warning "构建失败: $target"
        fi
    done
}

# 生成校验和
generate_checksums() {
    print_info "生成校验和..."
    
    cd "$RELEASE_DIR"
    sha256sum * > checksums.txt
    cat checksums.txt
    cd - > /dev/null
    
    print_success "校验和已生成"
}

# 创建 GitHub Release
create_release() {
    if ! command -v gh &> /dev/null; then
        print_warning "GitHub CLI 未安装，跳过自动创建 Release"
        print_info "请手动上传以下文件到 GitHub Release:"
        ls -la "$RELEASE_DIR"
        return
    fi
    
    # 检查是否已登录
    if ! gh auth status &> /dev/null; then
        print_error "GitHub CLI 未登录"
        print_info "请运行: gh auth login"
        exit 1
    fi
    
    print_info "创建 GitHub Release: $VERSION"
    
    # 检查 release 是否已存在
    if gh release view "$VERSION" &> /dev/null; then
        print_warning "Release $VERSION 已存在"
        read -p "是否更新现有 Release? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_info "请手动上传文件:"
            ls -la "$RELEASE_DIR"
            return
        fi
        
        # 上传文件到现有 release
        print_info "上传文件到现有 Release..."
        for file in "$RELEASE_DIR"/*; do
            if [ -f "$file" ]; then
                gh release upload "$VERSION" "$file" --clobber
            fi
        done
    else
        # 创建新 release
        gh release create "$VERSION" \
            --title "Release $VERSION" \
            --notes-file - \
            "$RELEASE_DIR"/* << EOF
## AgentLink CLI ${VERSION}

### 安装

**Linux/macOS:**
\`\`\`bash
curl -sSL https://raw.githubusercontent.com/agentlink/agentlink-cli/main/install.sh | sh
\`\`\`

**Windows:**
\`\`\`powershell
Invoke-WebRequest -Uri https://raw.githubusercontent.com/agentlink/agentlink-cli/main/install.ps1 -UseBasicParsing | Invoke-Expression
\`\`\`

### 预编译二进制

| 平台 | 架构 | 文件名 |
|------|------|--------|
| Linux | x86_64 | \`agentlink-linux-x86_64\` |
| Linux | x86_64 (musl) | \`agentlink-linux-x86_64-musl\` |
| Linux | ARM64 | \`agentlink-linux-aarch64\` |
| macOS | x86_64 | \`agentlink-macos-x86_64\` |
| macOS | ARM64 | \`agentlink-macos-aarch64\` |

### 校验和
见 \`checksums.txt\`
EOF
    fi
    
    print_success "GitHub Release 创建完成!"
    print_info "访问: https://github.com/agentlink/agentlink-cli/releases/tag/${VERSION}"
    
    # 触发 Homebrew Tap 自动更新
    print_info "触发 Homebrew Tap 自动更新..."
    if gh api repos/agentlink-im/homebrew-tap/dispatches \
        --input - <<< "{\"event_type\": \"release-published\", \"client_payload\": {\"version\": \"${VERSION}\"}}" 2>/dev/null; then
        print_success "Homebrew Tap 自动更新已触发"
        print_info "请检查: https://github.com/agentlink-im/homebrew-tap/actions"
    else
        print_warning "无法自动触发 Homebrew Tap 更新"
        print_info "请手动运行: cd ../homebrew-tap && ./update-formula.sh ${VERSION}"
    fi
}

# 主流程
main() {
    echo "========================================"
    echo "  AgentLink CLI 发布构建脚本"
    echo "========================================"
    echo ""
    
    check_dependencies
    
    print_info "构建目录: $RELEASE_DIR"
    
    # 清理旧构建
    rm -rf "$RELEASE_DIR"
    mkdir -p "$RELEASE_DIR"
    
    # 构建
    build_local
    cross_compile
    
    # 生成校验和
    generate_checksums
    
    echo ""
    print_success "所有构建完成!"
    print_info "构建文件:"
    ls -la "$RELEASE_DIR"
    
    echo ""
    read -p "是否创建/更新 GitHub Release? (Y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Nn]$ ]]; then
        print_info "请手动上传以下文件到 GitHub Release:"
        ls -la "$RELEASE_DIR"
        print_info "或者运行: gh release create ${VERSION} ${RELEASE_DIR}/*"
    else
        create_release
    fi
}

main "$@"
