# AgentLink CLI 自动安装脚本 (Windows PowerShell)
# 使用方法: Invoke-WebRequest -Uri https://raw.githubusercontent.com/agentlink/agentlink-cli/main/install.ps1 -UseBasicParsing | Invoke-Expression

$ErrorActionPreference = "Stop"

# 配置
$RepoOwner = "agentlink"
$RepoName = "agentlink-cli"
$GithubRepo = "${RepoOwner}/${RepoName}"
$BinaryName = "agentlink.exe"
$InstallDir = $env:INSTALL_DIR
if (-not $InstallDir) {
    $InstallDir = "$env:LOCALAPPDATA\Programs\agentlink"
}
$CargoInstallDir = "$env:USERPROFILE\.cargo\bin"

# 默认使用最新版本
$Version = $env:VERSION
if (-not $Version) {
    $Version = "latest"
}

# 颜色函数
function Write-Info($msg) {
    Write-Host "[INFO] $msg" -ForegroundColor Cyan
}

function Write-Success($msg) {
    Write-Host "[SUCCESS] $msg" -ForegroundColor Green
}

function Write-Warning($msg) {
    Write-Host "[WARNING] $msg" -ForegroundColor Yellow
}

function Write-Error($msg) {
    Write-Host "[ERROR] $msg" -ForegroundColor Red
}

# 检测架构
function Detect-Arch {
    $arch = $env:PROCESSOR_ARCHITECTURE
    switch ($arch) {
        "AMD64" { return "x86_64" }
        "ARM64" { return "aarch64" }
        "x86" { return "i686" }
        default { return "unknown" }
    }
}

# 获取最新版本
function Get-LatestVersion {
    try {
        $response = Invoke-WebRequest -Uri "https://api.github.com/repos/${GithubRepo}/releases/latest" -UseBasicParsing
        $content = $response.Content | ConvertFrom-Json
        return $content.tag_name
    } catch {
        Write-Error "无法获取最新版本号"
        exit 1
    }
}

# 确定资产名称
function Determine-AssetName($os, $arch) {
    $assetName = ""
    
    switch ($os) {
        "windows" {
            switch ($arch) {
                "x86_64" { $assetName = "agentlink-windows-x86_64.exe" }
                "aarch64" { $assetName = "agentlink-windows-aarch64.exe" }
            }
        }
    }
    
    return $assetName
}

# 下载二进制
function Download-Binary($version, $assetName, $outputPath) {
    $downloadUrl = ""
    if ($version -eq "latest") {
        $downloadUrl = "https://github.com/${GithubRepo}/releases/latest/download/${assetName}"
    } else {
        $downloadUrl = "https://github.com/${GithubRepo}/releases/download/${version}/${assetName}"
    }
    
    Write-Info "下载 ${assetName}..."
    
    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $outputPath -UseBasicParsing
        return $true
    } catch {
        return $false
    }
}

# 安装二进制
function Install-Binary($binaryPath) {
    # 创建安装目录
    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }
    
    Copy-Item $binaryPath "$InstallDir\$BinaryName" -Force
    Write-Success "已安装到 $InstallDir\$BinaryName"
    
    # 添加到 PATH
    $currentPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
    if ($currentPath -notlike "*$InstallDir*") {
        Write-Info "将 $InstallDir 添加到用户 PATH..."
        [System.Environment]::SetEnvironmentVariable("PATH", "$currentPath;$InstallDir", "User")
        $env:PATH = "$env:PATH;$InstallDir"
    }
}

# 从源码安装（备用）
function Install-FromSource {
    Write-Warning "预编译二进制不可用，将尝试从源码编译..."
    
    # 检查 Rust
    if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
        Write-Error "从源码编译需要 Rust 环境"
        Write-Host "请安装 Rust: https://rustup.rs/"
        exit 1
    }
    
    # 创建临时目录
    $tempDir = [System.IO.Path]::GetTempPath() + [System.Guid]::NewGuid().ToString()
    New-Item -ItemType Directory -Path $tempDir | Out-Null
    
    try {
        Write-Info "正在克隆仓库..."
        Set-Location $tempDir
        git clone --depth 1 "https://github.com/${GithubRepo}" agentlink-cli
        Set-Location agentlink-cli
        
        Write-Info "正在编译（这可能需要几分钟）..."
        cargo build --release
        
        $binaryPath = "target\release\$BinaryName"
        if (Test-Path $binaryPath) {
            Install-Binary $binaryPath
        } else {
            throw "编译失败：找不到二进制文件"
        }
    }
    finally {
        Set-Location $env:TEMP
        Remove-Item -Recurse -Force $tempDir -ErrorAction SilentlyContinue
    }
}

# 验证安装
function Test-Installation {
    if (Get-Command agentlink -ErrorAction SilentlyContinue) {
        $installPath = (Get-Command agentlink).Source
        Write-Success "agentlink-cli 安装成功!"
        Write-Info "安装位置: $installPath"
        Write-Info "版本信息:"
        & agentlink --version
        return $true
    } elseif (Test-Path "$InstallDir\$BinaryName") {
        Write-Success "agentlink-cli 安装成功!"
        Write-Info "安装位置: $InstallDir\$BinaryName"
        return $true
    }
    return $false
}

# 主安装流程
function Main {
    Write-Host "========================================"
    Write-Host "  AgentLink CLI 自动安装脚本"
    Write-Host "========================================"
    Write-Host ""
    
    $arch = Detect-Arch
    Write-Info "检测到架构: Windows ($arch)"
    
    # 获取版本号
    if ($Version -eq "latest") {
        Write-Info "正在获取最新版本..."
        $Version = Get-LatestVersion
    }
    Write-Info "安装版本: $Version"
    
    # 确定资产名称
    $assetName = Determine-AssetName "windows" $arch
    
    if (-not $assetName) {
        Write-Warning "当前平台没有预编译二进制"
        Install-FromSource
    } else {
        Write-Info "目标文件: $assetName"
        
        # 创建临时文件
        $tempFile = [System.IO.Path]::GetTempFileName()
        
        # 下载
        if (Download-Binary $Version $assetName $tempFile) {
            Install-Binary $tempFile
            Remove-Item $tempFile -ErrorAction SilentlyContinue
        } else {
            Remove-Item $tempFile -ErrorAction SilentlyContinue
            Write-Warning "下载预编译二进制失败"
            Install-FromSource
        }
    }
    
    Write-Host ""
    # 验证安装
    if (Test-Installation) {
        Write-Host ""
        Write-Host "========================================"
        Write-Success "安装完成!"
        Write-Host "========================================"
        Write-Host ""
        Write-Host "快速开始:"
        Write-Host "  1. 配置服务器地址（默认已配置为 beta）"
        Write-Host "     agentlink config set base_url https://beta-api.agentlink.chat/"
        Write-Host ""
        Write-Host "  2. 查看帮助"
        Write-Host "     agentlink --help"
        Write-Host ""
        Write-Host "  3. 查看版本"
        Write-Host "     agentlink --version"
        Write-Host ""
        Write-Host "更多信息请查看文档:"
        Write-Host "  https://github.com/${GithubRepo}"
    } else {
        Write-Error "安装可能未完成"
        exit 1
    }
}

# 运行主程序
Main
