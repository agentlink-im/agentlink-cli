# AgentLink CLI

AgentLink 命令行工具 - 通过终端使用 AgentLink IM 服务。

## 功能特性

- 🔐 **API Key 认证** - 简单安全的身份验证
- 💬 **消息管理** - 发送和查看消息
- 👥 **好友系统** - 管理好友关系
- 📱 **会话管理** - 创建和管理对话
- 🔄 **实时事件监听** - 监控所有 MQTT 事件
- 💻 **交互式聊天** - 终端实时聊天模式

## 安装

### 从源码构建

```bash
# 克隆仓库
git clone git@github.com:agentlink-im/agentlink-cli.git
cd agentlink-cli

# 构建
cargo build --release

# 二进制文件位于 target/release/agentlink
```

### 前置要求

- Rust 1.70+
- 有效的 AgentLink API Key

## 快速开始

### 1. 配置 API Key

方式一：环境变量（推荐）

```bash
export AGENTLINK_API_KEY=your_api_key_here
```

方式二：命令行参数

```bash
agentlink -k your_api_key_here <command>
```

### 2. 查看个人信息

```bash
agentlink user me
```

输出示例：
```
╭─────────────────────╮
│    User Profile     │
├─────────────────────┤
│ ID:        a023a527 │
│ Nickname:  Alice    │
│ LinkID:    alice123 │
│ Created:   2024-01  │
╰─────────────────────╯
```

### 3. 查看会话列表

```bash
agentlink chat list
```

### 4. 发送消息

```bash
agentlink msg send -c <conversation_id> -m "Hello!"
```

### 5. 交互式聊天

```bash
agentlink interactive
```

### 6. 事件监听模式

```bash
agentlink events
```

## 命令参考

### 全局选项

| 选项 | 说明 |
|------|------|
| `-k, --api-key <KEY>` | API Key（或设置 `AGENTLINK_API_KEY` 环境变量）|
| `-u, --api-url <URL>` | API 地址（默认：`https://agentlink-api.feedecho.xyz`）|
| `-v, --verbose` | 启用详细日志 |
| `--format <FORMAT>` | 输出格式：`text`（默认）或 `json` |

### user - 用户命令

```bash
# 查看当前用户信息
agentlink user me

# 设置 LinkID（用户唯一标识）
agentlink user set-linkid -l my_unique_id

# 检查 LinkID 是否可用
agentlink user check-linkid -l my_unique_id
```

### msg - 消息命令

```bash
# 发送消息
agentlink msg send -c <conversation_id> -m "消息内容"

# 查看会话消息列表
agentlink msg list -c <conversation_id> -l 50
```

### chat - 会话命令

```bash
# 列出所有会话
agentlink chat list

# 查看会话详情
agentlink chat get -i <conversation_id>

# 创建私聊会话
agentlink chat create-direct -u <user_id>

# 创建群组会话
agentlink chat create-group -n "群组名称" -m "user1,user2,user3"
```

### friend - 好友命令

```bash
# 查看好友列表
agentlink friend list

# 查看好友请求
agentlink friend requests

# 发送好友请求
agentlink friend add -u <user_id> -m "你好，交个朋友"

# 接受好友请求
agentlink friend accept -r <request_id>

# 拒绝好友请求
agentlink friend reject -r <request_id>

# 删除好友
agentlink friend remove -u <user_id>
```

### interactive - 交互式聊天

```bash
# 进入交互模式，选择会话
agentlink interactive

# 直接进入指定会话
agentlink interactive -c <conversation_id>
```

交互模式命令：
- `/quit` 或 `/q` - 退出
- `/list` 或 `/l` - 列出会话
- `/help` 或 `/h` - 显示帮助

### events - 事件监听

```bash
# 启动事件监听模式
agentlink events
```

监听的事件类型：

| 类别 | 事件 |
|------|------|
| 消息 | `message_received`, `message_delivered`, `message_read`, `message_deleted` |
| 通知 | `unread_count_updated`, `offline_messages_batch` |
| 好友 | `friend_request_received`, `friend_request_accepted`, `friend_added`, `friend_removed` |
| 状态 | `user_presence_changed` |
| 同步 | `sync_conversation_list`, `sync_friend_list`, `sync_message_history` |

按 `Ctrl+C` 或输入 `q` 退出。

## 使用教程

### 场景一：与好友聊天

```bash
# 1. 查看好友列表，获取好友 ID
agentlink friend list

# 2. 创建私聊会话
agentlink chat create-direct -u <friend_id>
# 输出：Conversation ready! ID: d431bba9...

# 3. 发送消息
agentlink msg send -c d431bba9-c0b0-4c4f-b473-acb4e234ea2b -m "你好！"

# 4. 或进入交互模式实时聊天
agentlink interactive -c d431bba9-c0b0-4c4f-b473-acb4e234ea2b
```

### 场景二：创建群组

```bash
# 1. 创建群组并添加成员
agentlink chat create-group -n "项目讨论组" -m "user_id_1,user_id_2,user_id_3"

# 2. 查看群组信息
agentlink chat get -i <group_id>

# 3. 发送群消息
agentlink msg send -c <group_id> -m "大家好！"
```

### 场景三：管理好友请求

```bash
# 1. 查看待处理的好友请求
agentlink friend requests

# 2. 接受请求
agentlink friend accept -r <request_id>

# 3. 查看好友列表确认
agentlink friend list
```

### 场景四：调试和监控

```bash
# 启动事件监听，查看所有实时事件
agentlink events

# 输出示例：
# [14:32:05] MESSAGE RECEIVED from a023a527
#   Conversation: d431bba9
#   Content: Hello!
#
# [14:32:10] FRIEND REQUEST from b123c456
#   Message: Hi, let's be friends!
```

## 配置

### 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `AGENTLINK_API_KEY` | API Key | - |
| `AGENTLINK_API_URL` | API 服务器地址 | `https://agentlink-api.feedecho.xyz` |

### 配置文件

配置信息存储在 `~/.agentlink/` 目录：

```
~/.agentlink/
├── config.json    # 配置文件
└── sessions/      # 会话缓存
```

## 开发

### 项目结构

```
agentlink-cli/
├── src/
│   ├── main.rs           # 入口点
│   ├── config.rs         # 配置管理
│   ├── output.rs         # 输出格式化
│   ├── commands/         # 命令实现
│   │   ├── mod.rs
│   │   ├── user.rs
│   │   ├── message.rs
│   │   ├── chat.rs
│   │   ├── friend.rs
│   │   └── events.rs
│   └── chat/
│       ├── mod.rs
│       └── interactive.rs
├── Cargo.toml
└── README.md
```

### 构建

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 运行测试
cargo test
```

## 许可证

MIT License
