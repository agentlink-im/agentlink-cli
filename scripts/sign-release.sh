#!/bin/bash
# AgentLink CLI 代码签名脚本
# 支持 macOS (codesign + notarytool) 和 Windows (signtool/osslsigncode)

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

# 配置
RELEASE_DIR="${1:-release}"

# 检查 macOS 签名环境
check_macos_signing_env() {
    if [ -z "$APPLE_DEVELOPER_ID" ]; then
        print_warning "未设置 APPLE_DEVELOPER_ID 环境变量"
        return 1
    fi
    if [ -z "$APPLE_TEAM_ID" ]; then
        print_warning "未设置 APPLE_TEAM_ID 环境变量"
        return 1
    fi
    if [ -z "$APPLE_APP_SPECIFIC_PASSWORD" ]; then
        print_warning "未设置 APPLE_APP_SPECIFIC_PASSWORD 环境变量"
        return 1
    fi
    return 0
}

# 检查 Windows 签名环境
check_windows_signing_env() {
    if [ -z "$WINDOWS_CERTIFICATE" ] && [ -z "$WINDOWS_CERTIFICATE_PATH" ]; then
        print_warning "未设置 WINDOWS_CERTIFICATE 或 WINDOWS_CERTIFICATE_PATH 环境变量"
        return 1
    fi
    if [ -z "$WINDOWS_CERTIFICATE_PASSWORD" ]; then
        print_warning "未设置 WINDOWS_CERTIFICATE_PASSWORD 环境变量"
        return 1
    fi
    return 0
}

# macOS 代码签名
sign_macos() {
    local binary="$1"
    
    if ! check_macos_signing_env; then
        print_warning "跳过 macOS 签名: ${binary}"
        return
    fi
    
    print_info "签名 macOS 二进制: ${binary}"
    
    # 使用 Developer ID 签名
    codesign --sign "$APPLE_DEVELOPER_ID" \
        --deep \
        --force \
        --verbose \
        --options runtime \
        --timestamp \
        "${binary}"
    
    # 验证签名
    codesign --verify --verbose "${binary}"
    
    print_success "macOS 签名完成: ${binary}"
}

# macOS 公证 (Notarization)
notarize_macos() {
    local binary="$1"
    local bundle_id="com.agentlink.cli"
    
    if ! check_macos_signing_env; then
        print_warning "跳过 macOS 公证: ${binary}"
        return
    fi
    
    print_info "公证 macOS 二进制: ${binary}"
    
    # 创建临时 zip 文件
    local temp_zip=$(mktemp).zip
    ditto -c -k --keepParent "${binary}" "${temp_zip}"
    
    # 提交公证
    local output
    output=$(xcrun notarytool submit "${temp_zip}" \
        --apple-id "$APPLE_DEVELOPER_ID" \
        --team-id "$APPLE_TEAM_ID" \
        --password "$APPLE_APP_SPECIFIC_PASSWORD" \
        --wait 2>&1)
    
    print_info "公证结果: ${output}"
    
    # 清理
    rm -f "${temp_zip}"
    
    print_success "macOS 公证完成: ${binary}"
}

# Windows 代码签名 (使用 osslsigncode)
sign_windows() {
    local binary="$1"
    
    if ! check_windows_signing_env; then
        print_warning "跳过 Windows 签名: ${binary}"
        return
    fi
    
    if ! command -v osslsigncode &> /dev/null; then
        print_warning "未安装 osslsigncode，跳过 Windows 签名"
        print_info "安装: brew install osslsigncode (macOS) 或 apt-get install osslsigncode (Linux)"
        return
    fi
    
    print_info "签名 Windows 二进制: ${binary}"
    
    local cert_file
    if [ -n "$WINDOWS_CERTIFICATE_PATH" ]; then
        cert_file="$WINDOWS_CERTIFICATE_PATH"
    else
        # 从环境变量解码证书
        cert_file=$(mktemp).pfx
        echo "$WINDOWS_CERTIFICATE" | base64 -d > "$cert_file"
    fi
    
    local signed_binary="${binary}.signed"
    
    # 签名
    osslsigncode sign \
        -pkcs12 "$cert_file" \
        -pass "$WINDOWS_CERTIFICATE_PASSWORD" \
        -n "AgentLink CLI" \
        -i "https://agentlink.chat" \
        -t "http://timestamp.digicert.com" \
        -in "${binary}" \
        -out "${signed_binary}"
    
    # 替换原文件
    mv "${signed_binary}" "${binary}"
    chmod +x "${binary}"
    
    # 清理临时证书文件
    if [ -z "$WINDOWS_CERTIFICATE_PATH" ]; then
        rm -f "$cert_file"
    fi
    
    print_success "Windows 签名完成: ${binary}"
}

# 主流程
main() {
    echo "========================================"
    echo "  AgentLink CLI 代码签名"
    echo "========================================"
    echo ""
    
    if [ ! -d "$RELEASE_DIR" ]; then
        print_error "发布目录不存在: $RELEASE_DIR"
        exit 1
    fi
    
    print_info "签名目录: $RELEASE_DIR"
    
    # 遍历所有二进制文件
    for binary in "$RELEASE_DIR"/agentlink-*; do
        if [ -f "$binary" ] && [ ! "$binary" =~ \.txt$ ]; then
            print_info "处理: $(basename "$binary")"
            
            if [[ "$binary" == *"macos"* ]]; then
                sign_macos "$binary"
                notarize_macos "$binary"
            elif [[ "$binary" == *"windows"* ]]; then
                sign_windows "$binary"
            else
                print_info "跳过签名 (Linux): $(basename "$binary")"
            fi
        fi
    done
    
    echo ""
    print_success "所有签名处理完成!"
}

main "$@"
