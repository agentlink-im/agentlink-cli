# AgentLink CLI Docker 镜像
# 用于 CI/CD 环境和容器化部署

# 构建阶段
FROM rust:1.75-slim-bookworm AS builder

WORKDIR /app

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

# 复制依赖文件
COPY Cargo.toml Cargo.lock ./

# 创建虚拟 main.rs 以缓存依赖
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# 复制源代码
COPY src ./src

# 构建（使用缓存的依赖）
RUN touch src/main.rs && \
    cargo build --release

# 运行阶段
FROM debian:bookworm-slim

LABEL maintainer="AgentLink Team <team@agentlink.chat>"
LABEL description="AgentLink CLI - AI Agent collaboration platform command line tool"
LABEL org.opencontainers.image.source="https://github.com/agentlink/agentlink-cli"
LABEL org.opencontainers.image.documentation="https://github.com/agentlink/agentlink-cli#readme"
LABEL org.opencontainers.image.licenses="MIT"

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 创建非 root 用户
RUN groupadd -r agentlink && useradd -r -g agentlink -m agentlink

# 复制二进制文件
COPY --from=builder /app/target/release/agentlink /usr/local/bin/agentlink

# 设置权限
RUN chmod +x /usr/local/bin/agentlink

# 切换到非 root 用户
USER agentlink

# 设置工作目录
WORKDIR /home/agentlink

# 创建配置目录
RUN mkdir -p /home/agentlink/.config/agentlink

# 默认环境变量
ENV AGENTLINK_BASE_URL=https://beta-api.agentlink.chat/
ENV AGENTLINK_CONFIG_DIR=/home/agentlink/.config/agentlink

# 验证安装
RUN agentlink --version

ENTRYPOINT ["agentlink"]
CMD ["--help"]
