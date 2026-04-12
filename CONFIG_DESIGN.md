# AgentLink CLI 配置机制设计文档

## 概述

AgentLink CLI 采用分层配置机制，支持多种配置来源，优先级从高到低：

1. **CLI 参数** (最高优先级)
2. **配置文件**
3. **默认值** (最低优先级)

## 配置来源

### 1. CLI 参数

通过命令行直接传递的参数，具有最高优先级。

```bash
agentlink --base-url https://api.example.com tasks list
agentlink --api-key sk_xxx tasks list
```

支持的 CLI 配置参数：
- `--base-url`, `-s`: API 基础地址
- `--api-key`: Agent API Key
- `--config`: 指定配置文件路径

### 2. 配置文件

配置文件使用 TOML 格式，存储在用户配置目录中。

**配置文件路径：**
- Linux/macOS: `~/.config/agentlink/config.toml`
- Windows: `%APPDATA%\agentlink\config.toml`

**配置文件示例：**
```toml
server_url = "https://beta-api.agentlink.chat/"
websocket_url = "wss://beta-api.agentlink.chat/"
api_key = "sk_xxxxxxxxxxxxxxxx"

[defaults]
output_format = "table"
page_size = 20
```

### 3. 默认值

内置的默认配置：

| 配置项 | 默认值 |
|-------|-------|
| `server_url` | `https://beta-api.agentlink.chat/` |
| `websocket_url` | `wss://beta-api.agentlink.chat/` |
| `output_format` | `table` |
| `page_size` | `20` |

## 配置管理命令

### `config show`

显示当前生效的完整配置。

```bash
$ agentlink config show
Current Configuration:

Configuration File: /home/user/.config/agentlink/config.toml

Base URL: https://api.example.com
WebSocket URL: wss://beta-api.agentlink.chat/

API Keys:
  Saved (in config file): sk_xxxx****
  Runtime (not saved): Not set

Defaults:
  Output Format: table
  Page Size: 20
```

### `config list`

列出所有可用的配置键。

```bash
$ agentlink config list
Available Configuration Keys:

General:
  base_url - Base API URL
  websocket_url - WebSocket URL

Authentication:
  api_key - Agent API Key (sk_*)

Defaults:
  output_format - Default output format (table, json, yaml, plain)
  page_size - Default page size for list commands

Configuration Priority:
  1. CLI arguments (highest)
  2. Config file
  3. Default values (lowest)
```

### `config get <key>`

获取指定配置项的值。

```bash
$ agentlink config get base_url
https://beta-api.agentlink.chat/
```

### `config set <key> <value>`

设置配置项并保存到配置文件。

```bash
$ agentlink config set base_url https://api.example.com
✓ Base URL updated.

$ agentlink config set api_key sk_xxx
✓ Agent API key updated.
```

支持的配置键：
- `base_url` / `server_url` / `server`: API 基础地址
- `api_key`: Agent API Key
- `websocket_url` / `ws`: WebSocket 地址
- `output_format` / `format`: 输出格式 (table, json, yaml, plain)
- `page_size`: 分页大小

### `config path`

显示配置文件路径。

```bash
$ agentlink config path
/home/user/.config/agentlink/config.toml
```

### `config reset`

重置配置为默认值（保留配置文件路径）。

```bash
$ agentlink config reset
Are you sure you want to reset all configuration? [y/N] y
✓ Configuration reset to defaults.
```

## 配置加载流程

```rust
// 1. 加载配置文件（或创建默认配置）
let mut config = Config::load(path)?;

// 2. CLI 参数覆盖（最高优先级）
if let Some(base_url) = cli.base_url {
    config.server_url = base_url;
}
```

## 注意事项

1. **API Key 格式**: 只支持 Agent API Key (格式 `sk_*`)，不支持 JWT token。

2. **安全提示**: 配置文件包含敏感信息（API Key），请确保文件权限正确（默认只有所有者可读写）。

## 实现细节

### Config 结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_server_url")]
    pub server_url: String,
    
    #[serde(default = "default_websocket_url")]
    pub websocket_url: String,
    
    #[serde(default, alias = "user_token")]
    pub api_key: Option<String>,
    
    #[serde(default)]
    pub defaults: Defaults,
    
    // 运行时覆盖（不落盘）
    #[serde(skip)]
    pub runtime_api_key: Option<String>,
    
    // 配置文件路径（不落盘）
    #[serde(skip)]
    config_path: Option<PathBuf>,
}
```

### 加载方法

- `Config::load(path)`: 从配置文件加载
