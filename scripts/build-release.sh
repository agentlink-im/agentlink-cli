#!/bin/bash
# AgentLink CLI 本地构建发布脚本
# 使用方法: ./scripts/build-release.sh [version] [--draft] [--skip-publish] [--yes]
# 示例: ./scripts/build-release.sh v0.1.0 --draft

set -euo pipefail

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

DEFAULT_REPOSITORY="agentlink-im/agentlink-cli"
DEFAULT_HOMEBREW_TAP_REPOSITORY="agentlink-im/homebrew-tap"
GITHUB_REPOSITORY="${AGENTLINK_GITHUB_REPOSITORY:-$DEFAULT_REPOSITORY}"
HOMEBREW_TAP_REPOSITORY="${AGENTLINK_HOMEBREW_TAP_REPOSITORY:-$DEFAULT_HOMEBREW_TAP_REPOSITORY}"
VERSION=""
CREATE_DRAFT_RELEASE=false
SKIP_BUILD=false
SKIP_PUBLISH=false
LOCAL_ONLY=false
AUTO_CONFIRM=false
TOKEN_SOURCE="gh auth login"

parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --draft)
                CREATE_DRAFT_RELEASE=true
                ;;
            --skip-build)
                SKIP_BUILD=true
                ;;
            --skip-publish)
                SKIP_PUBLISH=true
                ;;
            --local-only)
                LOCAL_ONLY=true
                ;;
            --yes|--non-interactive)
                AUTO_CONFIRM=true
                ;;
            --repo)
                shift
                if [[ $# -eq 0 ]]; then
                    print_error "--repo 需要提供 owner/repo"
                    exit 1
                fi
                GITHUB_REPOSITORY="$1"
                ;;
            --homebrew-tap-repo)
                shift
                if [[ $# -eq 0 ]]; then
                    print_error "--homebrew-tap-repo 需要提供 owner/repo"
                    exit 1
                fi
                HOMEBREW_TAP_REPOSITORY="$1"
                ;;
            --*)
                print_error "未知参数: $1"
                exit 1
                ;;
            *)
                if [[ -n "$VERSION" ]]; then
                    print_error "只允许传入一个版本号，收到多余参数: $1"
                    exit 1
                fi
                VERSION="$1"
                ;;
        esac
        shift
    done
}

read_version() {
    if [[ -z "$VERSION" ]]; then
        VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
        VERSION="v${VERSION}"
        print_info "未指定版本，使用 Cargo.toml 版本: $VERSION"
    elif [[ ! "$VERSION" =~ ^v ]]; then
        VERSION="v${VERSION}"
    fi
}

configure_github_auth() {
    if [[ -n "${AGENTLINK_RELEASE_TOKEN:-}" ]]; then
        export GH_TOKEN="${AGENTLINK_RELEASE_TOKEN}"
        TOKEN_SOURCE="AGENTLINK_RELEASE_TOKEN"
    elif [[ -n "${GITHUB_TOKEN:-}" ]]; then
        export GH_TOKEN="${GITHUB_TOKEN}"
        TOKEN_SOURCE="GITHUB_TOKEN"
    elif [[ -n "${GH_TOKEN:-}" ]]; then
        TOKEN_SOURCE="GH_TOKEN"
    fi
}

print_release_auth_help() {
    echo "  1. 创建一个可访问 ${GITHUB_REPOSITORY} 的 GitHub token"
    echo "     - 推荐：fine-grained PAT，生命周期 <= 366 天，并授予仓库访问权限"
    echo "     - 备选：classic PAT，至少包含 repo 权限"
    echo "  2. 在当前终端导出 token"
    echo "     export AGENTLINK_RELEASE_TOKEN=<your_token>"
    echo "  3. 重新执行"
    echo "     make release"
}

validate_release_access() {
    local error_file
    error_file=$(mktemp)

    if gh api "repos/${GITHUB_REPOSITORY}" --jq '.full_name' > /dev/null 2>"${error_file}"; then
        rm -f "${error_file}"
        print_success "GitHub Release 访问校验通过: ${GITHUB_REPOSITORY}"
        return
    fi

    local error_output
    error_output=$(cat "${error_file}")
    rm -f "${error_file}"

    print_error "无法访问 GitHub 仓库: ${GITHUB_REPOSITORY}"

    if [[ "${error_output}" == *"forbids access via a fine-grained personal access tokens if the token's lifetime is greater than 366 days"* ]]; then
        print_error "当前 token 不符合 agentlink-im 组织策略：fine-grained PAT 生命周期必须不超过 366 天"
        print_info "请改用合规 token 后重试："
        print_release_auth_help
        exit 1
    fi

    if [[ "${error_output}" == *"HTTP 403"* ]]; then
        print_error "当前 token 没有 ${GITHUB_REPOSITORY} 的发布权限"
        print_info "请改用具备仓库访问权限的 token："
        print_release_auth_help
        exit 1
    fi

    if [[ "${error_output}" == *"HTTP 404"* ]]; then
        print_error "仓库不存在，或当前 token 无法看到 ${GITHUB_REPOSITORY}"
        exit 1
    fi

    print_error "${error_output}"
    exit 1
}

release_exists() {
    gh api "repos/${GITHUB_REPOSITORY}/releases/tags/${VERSION}" > /dev/null 2>&1
}

should_publish_release() {
    if [[ "${SKIP_PUBLISH}" == "true" ]]; then
        return 1
    fi

    if [[ "${AUTO_CONFIRM}" == "true" ]] || [[ -n "${CI:-}" ]] || [[ "${AGENTLINK_RELEASE_YES:-}" == "1" ]]; then
        return 0
    fi

    read -p "是否创建/更新 GitHub Release? (Y/n) " -n 1 -r
    echo
    [[ ! "${REPLY}" =~ ^[Nn]$ ]]
}

parse_args "$@"
read_version
print_info "构建版本: $VERSION"
print_info "目标仓库: $GITHUB_REPOSITORY"

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
        if [[ "${SKIP_PUBLISH}" == "true" ]]; then
            print_warning "未安装 GitHub CLI (gh)，将跳过自动发布"
        else
            print_error "需要安装 GitHub CLI (gh) 才能自动创建 Release"
            print_info "安装指南: https://cli.github.com/"
            exit 1
        fi
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

build_release_notes() {
    local files=()
    local file

    while IFS= read -r file; do
        files+=("$file")
    done < <(find "$RELEASE_DIR" -maxdepth 1 -type f -printf '%f\n' | sort)

    cat << EOF
## AgentLink CLI ${VERSION}

### 安装

**Linux/macOS:**
\`\`\`bash
curl -sSL https://raw.githubusercontent.com/${GITHUB_REPOSITORY}/main/install.sh | sh
\`\`\`

**Windows:**
\`\`\`powershell
Invoke-WebRequest -Uri https://raw.githubusercontent.com/${GITHUB_REPOSITORY}/main/install.ps1 -UseBasicParsing | Invoke-Expression
\`\`\`

### 附件
EOF

    for file in "${files[@]}"; do
        echo "- \`${file}\`"
    done
}

has_complete_release_assets() {
    local expected_assets=(
        "agentlink-linux-x86_64"
        "agentlink-linux-aarch64"
        "agentlink-macos-x86_64"
        "agentlink-macos-aarch64"
        "checksums.txt"
    )
    local asset

    for asset in "${expected_assets[@]}"; do
        if [[ ! -f "${RELEASE_DIR}/${asset}" ]]; then
            return 1
        fi
    done

    return 0
}

# 创建 GitHub Release
create_release() {
    local release_url="https://github.com/${GITHUB_REPOSITORY}/releases/tag/${VERSION}"

    print_info "创建 GitHub Release: $VERSION"
    print_info "GitHub 认证来源: ${TOKEN_SOURCE}"

    if release_exists; then
        print_warning "Release $VERSION 已存在"
        if [[ "${AUTO_CONFIRM}" != "true" ]] && [[ -z "${CI:-}" ]] && [[ "${AGENTLINK_RELEASE_YES:-}" != "1" ]]; then
            read -p "是否更新现有 Release? (y/N) " -n 1 -r
            echo
            if [[ ! "${REPLY}" =~ ^[Yy]$ ]]; then
                print_info "请手动上传文件:"
                ls -la "$RELEASE_DIR"
                return
            fi
        fi
        
        # 上传文件到现有 release
        print_info "上传文件到现有 Release..."
        for file in "$RELEASE_DIR"/*; do
            if [ -f "$file" ]; then
                gh release upload "$VERSION" "$file" --repo "$GITHUB_REPOSITORY" --clobber
            fi
        done
    else
        local draft_args=()
        if [[ "${CREATE_DRAFT_RELEASE}" == "true" ]]; then
            draft_args+=(--draft)
        fi

        # 创建新 release
        gh release create "$VERSION" \
            --repo "$GITHUB_REPOSITORY" \
            --title "AgentLink CLI $VERSION" \
            "${draft_args[@]}" \
            --notes-file - \
            "$RELEASE_DIR"/* <<< "$(build_release_notes)"
    fi
    
    print_success "GitHub Release 创建完成!"
    print_info "访问: ${release_url}"

    if ! has_complete_release_assets; then
        print_warning "当前 Release 只包含本地平台或资产不完整，跳过 Homebrew Tap 自动更新"
        return
    fi

    # 触发 Homebrew Tap 自动更新
    print_info "触发 Homebrew Tap 自动更新..."
    if [[ -n "${HOMEBREW_TAP_TOKEN:-}" ]]; then
        if GH_TOKEN="${HOMEBREW_TAP_TOKEN}" gh api "repos/${HOMEBREW_TAP_REPOSITORY}/dispatches" \
            --method POST \
            --input - <<< "{\"event_type\": \"release-published\", \"client_payload\": {\"version\": \"${VERSION}\"}}" 2>/dev/null; then
            print_success "Homebrew Tap 自动更新已触发"
            print_info "请检查: https://github.com/${HOMEBREW_TAP_REPOSITORY}/actions"
        else
            print_warning "无法自动触发 Homebrew Tap 更新"
            print_info "请手动运行: cd ../homebrew-tap && ./update-formula.sh ${VERSION}"
        fi
    elif gh api "repos/${HOMEBREW_TAP_REPOSITORY}/dispatches" \
        --method POST \
        --input - <<< "{\"event_type\": \"release-published\", \"client_payload\": {\"version\": \"${VERSION}\"}}" 2>/dev/null; then
        print_success "Homebrew Tap 自动更新已触发"
        print_info "请检查: https://github.com/${HOMEBREW_TAP_REPOSITORY}/actions"
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
    configure_github_auth
    
    print_info "构建目录: $RELEASE_DIR"

    if [[ "${SKIP_PUBLISH}" != "true" ]]; then
        validate_release_access
    fi

    # 清理旧构建
    rm -rf "$RELEASE_DIR"
    mkdir -p "$RELEASE_DIR"

    if [[ "${SKIP_BUILD}" != "true" ]]; then
        # 构建
        build_local
        if [[ "${LOCAL_ONLY}" == "true" ]]; then
            print_info "仅发布本地平台，跳过交叉编译"
        else
            cross_compile
        fi

        # 生成校验和
        generate_checksums
    else
        print_info "跳过构建阶段"
    fi

    echo ""
    print_success "所有构建完成!"
    print_info "构建文件:"
    ls -la "$RELEASE_DIR"
    
    if ! should_publish_release; then
        echo ""
        print_info "请手动上传以下文件到 GitHub Release:"
        ls -la "$RELEASE_DIR"
        print_info "或者运行: gh release create ${VERSION} ${RELEASE_DIR}/* --repo ${GITHUB_REPOSITORY}"
    else
        create_release
    fi
}

main "$@"
