#!/bin/sh
# AgentLink CLI 自动安装脚本
# 使用方法: curl -sSL https://raw.githubusercontent.com/agentlink-im/agentlink-cli/main/install.sh | sh

set -eu

REPO_OWNER="agentlink-im"
REPO_NAME="agentlink-cli"
GITHUB_REPO="${REPO_OWNER}/${REPO_NAME}"
BINARY_NAME="agentlink"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
CARGO_INSTALL_DIR="${CARGO_HOME:-$HOME/.cargo}/bin"
VERSION="${VERSION:-latest}"
INSTALL_LOCATION=""

if [ -t 1 ]; then
    RED="$(printf '\033[0;31m')"
    GREEN="$(printf '\033[0;32m')"
    YELLOW="$(printf '\033[1;33m')"
    BLUE="$(printf '\033[0;34m')"
    NC="$(printf '\033[0m')"
else
    RED=""
    GREEN=""
    YELLOW=""
    BLUE=""
    NC=""
fi

print_info() {
    printf '%s[INFO]%s %s\n' "$BLUE" "$NC" "$1"
}

print_success() {
    printf '%s[SUCCESS]%s %s\n' "$GREEN" "$NC" "$1"
}

print_warning() {
    printf '%s[WARNING]%s %s\n' "$YELLOW" "$NC" "$1"
}

print_error() {
    printf '%s[ERROR]%s %s\n' "$RED" "$NC" "$1" >&2
}

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

detect_os() {
    os="$(uname -s)"
    case "$os" in
        Linux*) printf 'linux\n' ;;
        Darwin*) printf 'macos\n' ;;
        CYGWIN*|MINGW*|MSYS*) printf 'windows\n' ;;
        *) printf 'unknown\n' ;;
    esac
}

detect_arch() {
    arch="$(uname -m)"
    case "$arch" in
        x86_64|amd64) printf 'x86_64\n' ;;
        arm64|aarch64) printf 'aarch64\n' ;;
        armv7l) printf 'armv7\n' ;;
        i386|i686) printf 'i686\n' ;;
        *) printf 'unknown\n' ;;
    esac
}

download_to_file() {
    url="$1"
    output_path="$2"

    if command_exists curl; then
        curl -fsSL -o "$output_path" "$url"
        return 0
    fi

    if command_exists wget; then
        wget -q -O "$output_path" "$url"
        return 0
    fi

    print_error "需要 curl 或 wget"
    return 1
}

checksum_file() {
    file_path="$1"

    if command_exists sha256sum; then
        sha256sum "$file_path" | awk '{print $1}'
        return 0
    fi

    if command_exists shasum; then
        shasum -a 256 "$file_path" | awk '{print $1}'
        return 0
    fi

    return 1
}

get_latest_version() {
    metadata_file="$(mktemp)"
    trap 'rm -f "$metadata_file"' EXIT INT TERM

    download_to_file "https://api.github.com/repos/${GITHUB_REPO}/releases/latest" "$metadata_file"
    latest_version="$(grep -o '"tag_name": "[^"]*"' "$metadata_file" | head -1 | cut -d'"' -f4)"

    rm -f "$metadata_file"
    trap - EXIT INT TERM

    if [ -z "$latest_version" ]; then
        print_error "无法获取最新版本号"
        exit 1
    fi

    printf '%s\n' "$latest_version"
}

determine_asset_name() {
    os="$1"
    arch="$2"

    case "$os" in
        linux)
            case "$arch" in
                x86_64) printf 'agentlink-linux-x86_64\n' ;;
                aarch64) printf 'agentlink-linux-aarch64\n' ;;
                armv7) printf 'agentlink-linux-armv7\n' ;;
                *) printf '\n' ;;
            esac
            ;;
        macos)
            case "$arch" in
                x86_64) printf 'agentlink-macos-x86_64\n' ;;
                aarch64) printf 'agentlink-macos-aarch64\n' ;;
                *) printf '\n' ;;
            esac
            ;;
        *)
            printf '\n'
            ;;
    esac
}

download_binary() {
    version="$1"
    asset_name="$2"
    output_path="$3"

    if [ "$version" = "latest" ]; then
        download_url="https://github.com/${GITHUB_REPO}/releases/latest/download/${asset_name}"
    else
        download_url="https://github.com/${GITHUB_REPO}/releases/download/${version}/${asset_name}"
    fi

    print_info "下载 ${asset_name}..."
    download_to_file "$download_url" "$output_path"
}

verify_checksum() {
    version="$1"
    binary_path="$2"
    asset_name="$3"

    if ! command_exists curl && ! command_exists wget; then
        return 0
    fi

    print_info "验证校验和..."

    if [ "$version" = "latest" ]; then
        checksum_url="https://github.com/${GITHUB_REPO}/releases/latest/download/checksums.txt"
    else
        checksum_url="https://github.com/${GITHUB_REPO}/releases/download/${version}/checksums.txt"
    fi

    checksum_file_path="$(mktemp)"
    trap 'rm -f "$checksum_file_path"' EXIT INT TERM

    if ! download_to_file "$checksum_url" "$checksum_file_path" >/dev/null 2>&1; then
        rm -f "$checksum_file_path"
        trap - EXIT INT TERM
        return 0
    fi

    expected_checksum="$(grep "${asset_name}\$" "$checksum_file_path" | awk '{print $1}')"
    rm -f "$checksum_file_path"
    trap - EXIT INT TERM

    if [ -z "$expected_checksum" ]; then
        return 0
    fi

    actual_checksum="$(checksum_file "$binary_path" || true)"
    if [ -z "$actual_checksum" ]; then
        print_warning "系统缺少 sha256 校验工具，跳过校验"
        return 0
    fi

    if [ "$expected_checksum" = "$actual_checksum" ]; then
        print_success "校验和验证通过"
    else
        print_warning "校验和不匹配，但将继续安装"
    fi
}

install_binary() {
    binary_path="$1"
    target_path="${INSTALL_DIR}/${BINARY_NAME}"

    if [ -w "$INSTALL_DIR" ] || [ "$(id -u)" -eq 0 ]; then
        cp "$binary_path" "$target_path"
        chmod +x "$target_path"
        INSTALL_LOCATION="$target_path"
        print_success "已安装到 $target_path"
        return 0
    fi

    if command_exists sudo; then
        print_info "需要 sudo 权限来安装到 $INSTALL_DIR"
        sudo cp "$binary_path" "$target_path"
        sudo chmod +x "$target_path"
        INSTALL_LOCATION="$target_path"
        print_success "已安装到 $target_path"
        return 0
    fi

    mkdir -p "$CARGO_INSTALL_DIR"
    target_path="${CARGO_INSTALL_DIR}/${BINARY_NAME}"
    cp "$binary_path" "$target_path"
    chmod +x "$target_path"
    INSTALL_LOCATION="$target_path"
    print_success "已安装到 $target_path"

    case ":$PATH:" in
        *":$CARGO_INSTALL_DIR:"*) ;;
        *)
            print_warning "$CARGO_INSTALL_DIR 不在 PATH 中"
            printf '请添加以下行到你的 shell 配置文件 (~/.bashrc, ~/.zshrc 等):\n'
            printf '  export PATH="%s:$PATH"\n' "$CARGO_INSTALL_DIR"
            ;;
    esac
}

install_from_source() {
    print_warning "预编译二进制不可用，将尝试从源码编译..."

    if ! command_exists git || ! command_exists rustc || ! command_exists cargo; then
        print_error "从源码编译需要 git、rustc 和 cargo"
        print_info "请安装 Rust: https://rustup.rs/"
        exit 1
    fi

    temp_dir="$(mktemp -d)"
    trap 'rm -rf "$temp_dir"' EXIT INT TERM

    print_info "正在克隆仓库..."
    (
        cd "$temp_dir"
        git clone --depth 1 "https://github.com/${GITHUB_REPO}" agentlink-cli
        cd agentlink-cli
        print_info "正在编译（这可能需要几分钟）..."
        cargo build --release
    )

    binary_path="${temp_dir}/agentlink-cli/target/release/${BINARY_NAME}"
    if [ ! -f "$binary_path" ]; then
        print_error "编译失败：找不到二进制文件"
        exit 1
    fi

    install_binary "$binary_path"

    rm -rf "$temp_dir"
    trap - EXIT INT TERM
}

verify_installation() {
    if ! command_exists "$BINARY_NAME"; then
        if [ -n "$INSTALL_LOCATION" ] && [ -x "$INSTALL_LOCATION" ]; then
            print_success "agentlink-cli 安装成功!"
            print_info "安装位置: $INSTALL_LOCATION"
            print_info "版本信息:"
            "$INSTALL_LOCATION" --version
            return 0
        fi

        print_error "验证安装失败"
        return 1
    fi

    install_path="$(command -v "$BINARY_NAME")"
    print_success "agentlink-cli 安装成功!"
    print_info "安装位置: $install_path"
    print_info "版本信息:"
    "$BINARY_NAME" --version
    return 0
}

print_help() {
    cat << EOF
AgentLink CLI 安装脚本

用法:
  curl -sSL https://raw.githubusercontent.com/${GITHUB_REPO}/main/install.sh | sh

选项:
  -v, --version <version>   指定安装版本 (默认: latest)
  --install-dir <dir>       指定安装目录 (默认: /usr/local/bin)
  -h, --help                显示此帮助信息

环境变量:
  VERSION                   指定安装版本
  INSTALL_DIR               指定安装目录
EOF
}

main() {
    printf '========================================\n'
    printf '  AgentLink CLI 自动安装脚本\n'
    printf '========================================\n\n'

    os="$(detect_os)"
    arch="$(detect_arch)"

    print_info "检测到系统: $os ($arch)"

    if ! command_exists curl && ! command_exists wget; then
        print_error "需要 curl 或 wget"
        exit 1
    fi

    if [ "$VERSION" = "latest" ]; then
        print_info "正在获取最新版本..."
        VERSION="$(get_latest_version)"
    fi
    print_info "安装版本: $VERSION"

    asset_name="$(determine_asset_name "$os" "$arch")"
    if [ -z "$asset_name" ]; then
        print_warning "当前平台 ($os/$arch) 没有预编译二进制"
        install_from_source
    else
        print_info "目标文件: $asset_name"
        temp_dir="$(mktemp -d)"
        trap 'rm -rf "$temp_dir"' EXIT INT TERM
        temp_binary="${temp_dir}/${BINARY_NAME}"

        if download_binary "$VERSION" "$asset_name" "$temp_binary"; then
            verify_checksum "$VERSION" "$temp_binary" "$asset_name"
            chmod +x "$temp_binary"
            install_binary "$temp_binary"
            rm -rf "$temp_dir"
            trap - EXIT INT TERM
        else
            print_warning "下载预编译二进制失败"
            rm -rf "$temp_dir"
            trap - EXIT INT TERM
            install_from_source
        fi
    fi

    printf '\n'
    if verify_installation; then
        printf '\n'
        printf '========================================\n'
        print_success "安装完成!"
        printf '========================================\n\n'
        printf '快速开始:\n'
        printf '  1. 配置服务器地址（默认已配置为 beta）\n'
        printf '     agentlink config set base_url https://beta-api.agentlink.chat/\n\n'
        printf '  2. 查看帮助\n'
        printf '     agentlink --help\n\n'
        printf '  3. 查看版本\n'
        printf '     agentlink --version\n\n'
        printf '更多信息请查看文档:\n'
        printf '  https://github.com/%s\n' "$GITHUB_REPO"
    else
        print_error "安装可能未完成"
        exit 1
    fi
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --version|-v)
            if [ "$#" -lt 2 ]; then
                print_error "--version 需要提供版本号"
                exit 1
            fi
            VERSION="$2"
            shift 2
            ;;
        --install-dir)
            if [ "$#" -lt 2 ]; then
                print_error "--install-dir 需要提供目录"
                exit 1
            fi
            INSTALL_DIR="$2"
            shift 2
            ;;
        --help|-h)
            print_help
            exit 0
            ;;
        *)
            shift
            ;;
    esac
done

main
