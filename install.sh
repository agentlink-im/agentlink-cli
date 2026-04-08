#!/bin/bash
# AgentLink CLI 自动安装脚本
# 使用方法: curl -sSL https://raw.githubusercontent.com/agentlink/agentlink-cli/main/install.sh | sh

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置
REPO_OWNER="agentlink"
REPO_NAME="agentlink-cli"
GITHUB_REPO="${REPO_OWNER}/${REPO_NAME}"
BINARY_NAME="agentlink"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
CARGO_INSTALL_DIR="${CARGO_HOME:-$HOME/.cargo}/bin"

# 默认使用最新版本，可以通过环境变量覆盖
VERSION="${VERSION:-latest}"

# 打印函数
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检测操作系统
detect_os() {
    local os
    os=$(uname -s)
    case "$os" in
        Linux*)     echo "linux";;
        Darwin*)    echo "macos";;
        CYGWIN*|MINGW*|MSYS*) echo "windows";;
        *)          echo "unknown";;
    esac
}

# 检测架构
detect_arch() {
    local arch
    arch=$(uname -m)
    case "$arch" in
        x86_64|amd64)   echo "x86_64";;
        arm64|aarch64)  echo "aarch64";;
        armv7l)         echo "armv7";;
        i386|i686)      echo "i686";;
        *)              echo "unknown";;
    esac
}

# 检查命令是否存在
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# 获取最新版本号
get_latest_version() {
    local version
    version=$(curl -sL "https://api.github.com/repos/${GITHUB_REPO}/releases/latest" | grep -o '"tag_name": "[^"]*"' | cut -d'"' -f4)
    if [ -z "$version" ]; then
        print_error "无法获取最新版本号"
        exit 1
    fi
    echo "$version"
}

# 确定下载文件名
determine_asset_name() {
    local os="$1"
    local arch="$2"
    local asset_name=""
    
    case "$os" in
        linux)
            case "$arch" in
                x86_64)     asset_name="agentlink-linux-x86_64";;
                aarch64)    asset_name="agentlink-linux-aarch64";;
                armv7)      asset_name="agentlink-linux-armv7";;
                *)          asset_name="";;
            esac
            ;;
        macos)
            case "$arch" in
                x86_64)     asset_name="agentlink-macos-x86_64";;
                aarch64)    asset_name="agentlink-macos-aarch64";;
                *)          asset_name="";;
            esac
            ;;
        *)
            asset_name=""
            ;;
    esac
    
    echo "$asset_name"
}

# 下载预编译二进制
download_binary() {
    local version="$1"
    local asset_name="$2"
    local output_path="$3"
    
    local download_url
    if [ "$version" = "latest" ]; then
        download_url="https://github.com/${GITHUB_REPO}/releases/latest/download/${asset_name}"
    else
        download_url="https://github.com/${GITHUB_REPO}/releases/download/${version}/${asset_name}"
    fi
    
    print_info "下载 ${asset_name}..."
    
    if command_exists curl; then
        curl -fsSL -o "$output_path" "$download_url"
    elif command_exists wget; then
        wget -q -O "$output_path" "$download_url"
    else
        print_error "需要 curl 或 wget 来下载二进制文件"
        exit 1
    fi
}

# 验证校验和
verify_checksum() {
    local version="$1"
    local binary_path="$2"
    local asset_name="$3"
    
    print_info "验证校验和..."
    
    local checksum_url
    if [ "$version" = "latest" ]; then
        checksum_url="https://github.com/${GITHUB_REPO}/releases/latest/download/checksums.txt"
    else
        checksum_url="https://github.com/${GITHUB_REPO}/releases/download/${version}/checksums.txt"
    fi
    
    local temp_checksum_file
    temp_checksum_file=$(mktemp)
    
    if curl -fsSL -o "$temp_checksum_file" "$checksum_url" 2>/dev/null; then
        local expected_checksum
        expected_checksum=$(grep "${asset_name}$" "$temp_checksum_file" | awk '{print $1}')
        
        if [ -n "$expected_checksum" ]; then
            local actual_checksum
            actual_checksum=$(sha256sum "$binary_path" | awk '{print $1}')
            
            if [ "$expected_checksum" = "$actual_checksum" ]; then
                print_success "校验和验证通过"
            else
                print_warning "校验和不匹配，但将继续安装"
            fi
        fi
    fi
    
    rm -f "$temp_checksum_file"
}

# 安装二进制文件
install_binary() {
    local binary_path="$1"
    local target_name="$BINARY_NAME"
    
    # 检查是否有写入权限
    if [ -w "$INSTALL_DIR" ] || [ "$EUID" -eq 0 ]; then
        cp "$binary_path" "$INSTALL_DIR/$target_name"
        chmod +x "$INSTALL_DIR/$target_name"
        print_success "已安装到 $INSTALL_DIR/$target_name"
        INSTALL_LOCATION="$INSTALL_DIR/$target_name"
    else
        # 尝试使用 sudo
        if command_exists sudo; then
            print_info "需要 sudo 权限来安装到 $INSTALL_DIR"
            sudo cp "$binary_path" "$INSTALL_DIR/$target_name"
            sudo chmod +x "$INSTALL_DIR/$target_name"
            print_success "已安装到 $INSTALL_DIR/$target_name"
            INSTALL_LOCATION="$INSTALL_DIR/$target_name"
        else
            # 安装到 cargo bin 目录
            mkdir -p "$CARGO_INSTALL_DIR"
            cp "$binary_path" "$CARGO_INSTALL_DIR/$target_name"
            chmod +x "$CARGO_INSTALL_DIR/$target_name"
            print_success "已安装到 $CARGO_INSTALL_DIR/$target_name"
            INSTALL_LOCATION="$CARGO_INSTALL_DIR/$target_name"
            
            # 检查 PATH
            if [[ ":$PATH:" != *":$CARGO_INSTALL_DIR:"* ]]; then
                print_warning "$CARGO_INSTALL_DIR 不在 PATH 中"
                echo "请添加以下行到你的 shell 配置文件 (~/.bashrc, ~/.zshrc 等):"
                echo "  export PATH=\"$CARGO_INSTALL_DIR:\$PATH\""
            fi
        fi
    fi
}

# 从源码安装（备用方案）
install_from_source() {
    print_warning "预编译二进制不可用，将尝试从源码编译..."
    
    # 检查 Rust 环境
    if ! command_exists rustc || ! command_exists cargo; then
        print_error "从源码编译需要 Rust 环境"
        print_info "请安装 Rust: https://rustup.rs/"
        exit 1
    fi
    
    # 创建临时目录
    local temp_dir
    temp_dir=$(mktemp -d)
    cd "$temp_dir"
    
    # 克隆仓库
    print_info "正在克隆仓库..."
    git clone --depth 1 "https://github.com/${GITHUB_REPO}" agentlink-cli
    cd agentlink-cli
    
    # 编译发布版本
    print_info "正在编译（这可能需要几分钟）..."
    cargo build --release
    
    # 安装二进制文件
    local binary_path="target/release/$BINARY_NAME"
    
    if [ -f "$binary_path" ]; then
        install_binary "$binary_path"
    else
        print_error "编译失败：找不到二进制文件"
        exit 1
    fi
    
    # 清理
    cd /
    rm -rf "$temp_dir"
}

# 验证安装
verify_installation() {
    if command_exists "$BINARY_NAME"; then
        local install_path
        install_path=$(command -v "$BINARY_NAME")
        print_success "agentlink-cli 安装成功!"
        print_info "安装位置: $install_path"
        print_info "版本信息:"
        "$BINARY_NAME" --version
        return 0
    else
        print_error "验证安装失败"
        return 1
    fi
}

# 主安装流程
main() {
    echo "========================================"
    echo "  AgentLink CLI 自动安装脚本"
    echo "========================================"
    echo ""
    
    local os
    local arch
    os=$(detect_os)
    arch=$(detect_arch)
    
    print_info "检测到系统: $os ($arch)"
    
    # 检查依赖
    if ! command_exists curl && ! command_exists wget; then
        print_error "需要 curl 或 wget"
        exit 1
    fi
    
    # 获取版本号
    if [ "$VERSION" = "latest" ]; then
        print_info "正在获取最新版本..."
        VERSION=$(get_latest_version)
    fi
    print_info "安装版本: $VERSION"
    
    # 确定资产名称
    local asset_name
    asset_name=$(determine_asset_name "$os" "$arch")
    
    if [ -z "$asset_name" ]; then
        print_warning "当前平台 ($os/$arch) 没有预编译二进制"
        install_from_source
    else
        print_info "目标文件: $asset_name"
        
        # 创建临时目录
        local temp_dir
        temp_dir=$(mktemp -d)
        local temp_binary="$temp_dir/$BINARY_NAME"
        
        # 下载二进制
        if download_binary "$VERSION" "$asset_name" "$temp_binary"; then
            # 验证校验和
            verify_checksum "$VERSION" "$temp_binary" "$asset_name"
            
            # 安装
            chmod +x "$temp_binary"
            install_binary "$temp_binary"
            
            # 清理
            rm -rf "$temp_dir"
        else
            print_warning "下载预编译二进制失败"
            rm -rf "$temp_dir"
            install_from_source
        fi
    fi
    
    echo ""
    # 验证安装
    if verify_installation; then
        echo ""
        echo "========================================"
        print_success "安装完成!"
        echo "========================================"
        echo ""
        echo "快速开始:"
        echo "  1. 配置服务器地址（默认已配置为 beta）"
        echo "     agentlink config set base_url https://beta-api.agentlink.chat/"
        echo ""
        echo "  2. 查看帮助"
        echo "     agentlink --help"
        echo ""
        echo "  3. 查看版本"
        echo "     agentlink --version"
        echo ""
        echo "更多信息请查看文档:"
        echo "  https://github.com/${GITHUB_REPO}"
    else
        print_error "安装可能未完成"
        exit 1
    fi
}

# 处理命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        --version|-v)
            VERSION="$2"
            shift 2
            ;;
        --install-dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        --help|-h)
            echo "AgentLink CLI 安装脚本"
            echo ""
            echo "用法:"
            echo "  curl -sSL https://raw.githubusercontent.com/${GITHUB_REPO}/main/install.sh | sh"
            echo ""
            echo "选项:"
            echo "  -v, --version <version>   指定安装版本 (默认: latest)"
            echo "  --install-dir <dir>       指定安装目录 (默认: /usr/local/bin)"
            echo "  -h, --help                显示此帮助信息"
            echo ""
            echo "环境变量:"
            echo "  VERSION                   指定安装版本"
            echo "  INSTALL_DIR               指定安装目录"
            exit 0
            ;;
        *)
            shift
            ;;
    esac
done

# 运行主程序
main
