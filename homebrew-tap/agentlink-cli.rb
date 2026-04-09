# 本地开发用公式
# 正式公式请见: https://github.com/agentlink-im/homebrew-tap

class AgentlinkCli < Formula
  desc "CLI tool for AgentLink - AI Agent collaboration platform"
  homepage "https://github.com/agentlink-im/agentlink-cli"
  version "0.1.1"
  license "MIT"

  # 根据平台自动选择二进制文件
  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/agentlink-im/agentlink-cli/releases/download/v#{version}/agentlink-macos-aarch64"
      sha256 "PLACEHOLDER_SHA256_MACOS_AARCH64"
    else
      url "https://github.com/agentlink-im/agentlink-cli/releases/download/v#{version}/agentlink-macos-x86_64"
      sha256 "PLACEHOLDER_SHA256_MACOS_X86_64"
    end
  elsif OS.linux?
    if Hardware::CPU.arm?
      url "https://github.com/agentlink-im/agentlink-cli/releases/download/v#{version}/agentlink-linux-aarch64"
      sha256 "PLACEHOLDER_SHA256_LINUX_AARCH64"
    else
      url "https://github.com/agentlink-im/agentlink-cli/releases/download/v#{version}/agentlink-linux-x86_64"
      sha256 "PLACEHOLDER_SHA256_LINUX_X86_64"
    end
  end

  def install
    bin.install Dir["agentlink-*"].first => "agentlink"
  end

  test do
    # 测试版本命令
    assert_match version.to_s, shell_output("#{bin}/agentlink --version")
    
    # 测试帮助命令
    assert_match "AgentLink CLI", shell_output("#{bin}/agentlink --help")
  end
end
