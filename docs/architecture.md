# nexus-gate 架构说明

## 项目定位

**nexus-gate** 是一个高性能 AI API 指纹中转网关，核心理念类似"指纹浏览器"——每个发往上游的请求都可以携带独立的 TLS/HTTP/Header 指纹，使上游 API 提供商无法通过网络指纹特征识别中转行为。

### 与现有方案的本质区别

| 维度 | claude-relay-service | sub2api | **nexus-gate** |
|------|---------------------|---------|----------------|
| 语言 | Node.js / Express | Go / Gin | **Rust / Axum** |
| 存储 | Redis-only | PostgreSQL + Redis (Ent ORM) | **PostgreSQL + Redis (sqlx raw SQL)** |
| 指纹控制 | 无 (固定 User-Agent) | 有限 (TLS profile 表) | **完整指纹引擎 (TLS+HTTP/2+Header)** |
| API Key 前缀 | `cr_` | 无固定前缀 | `nk-` |
| 错误码体系 | E001-E015 | code/message | **标准 Anthropic/OpenAI 错误格式** |
| 可识别指纹 | 高 (大量硬编码标识) | 中 (Go module/Docker标签) | **极低 (零项目标识泄露)** |

---

## 架构分层

```
┌─────────────────────────────────────────────────────────┐
│                    API Layer (src/api/)                   │
│  claude.rs │ openai.rs │ gemini.rs │ admin.rs │ health.rs│
├─────────────────────────────────────────────────────────┤
│               Middleware (src/middleware/)                │
│         auth.rs │ rate_limit.rs │ request_id.rs          │
├─────────────────────────────────────────────────────────┤
│                Service Layer (src/service/)               │
│  relay.rs │ scheduler.rs │ session.rs │ cost.rs          │
│  account.rs │ apikey.rs                                  │
├─────────────────────────────────────────────────────────┤
│            Fingerprint Engine (src/fingerprint/)          │
│  engine.rs │ profile.rs │ tls.rs │ headers.rs            │
├─────────────────────────────────────────────────────────┤
│              Repository Layer (src/repo/)                 │
│     account.rs │ apikey.rs │ usage.rs │ cache.rs         │
├─────────────────────────────────────────────────────────┤
│              Data Models (src/model/)                     │
│     account.rs │ apikey.rs │ usage.rs │ types.rs         │
├─────────────────────────────────────────────────────────┤
│              Utilities (src/util/)                        │
│     crypto.rs │ stream.rs │ proxy.rs │ mask.rs           │
├─────────────────────────────────────────────────────────┤
│           Infrastructure                                  │
│     PostgreSQL │ Redis │ reqwest HTTP client              │
└─────────────────────────────────────────────────────────┘
```

### 层级职责

| 层 | 目录 | 职责 | 测试重点 |
|----|------|------|----------|
| **API** | `src/api/` | HTTP 路由 + 请求/响应序列化 | 请求解析、响应格式 |
| **Middleware** | `src/middleware/` | 认证、限流、请求追踪 | Key 提取、hash、限流逻辑 |
| **Service** | `src/service/` | 核心业务：中转、调度、会话、计费 | 调度算法、会话 hash、成本计算 |
| **Fingerprint** | `src/fingerprint/` | TLS/HTTP/Header 指纹伪装 | Profile 序列化、JA3 计算、Header 排序 |
| **Repository** | `src/repo/` | 数据访问 (SQL + Redis) | SQL 正确性 |
| **Model** | `src/model/` | 数据实体定义 | 序列化/反序列化 |
| **Util** | `src/util/` | 加密、流处理、代理、脱敏 | 加密回环、代理格式、脱敏覆盖 |

---

## 核心请求流程

```
客户端 (nk-前缀 Key)
  │
  ▼
Axum Router ──► request_id middleware (注入唯一 ID)
  │
  ▼
auth middleware ──► 提取 API Key → SHA-256 hash → 查 PostgreSQL
  │                  验证 status / expires_at / permissions
  ▼
rate_limit middleware ──► Redis INCR 滑动窗口计数
  │
  ▼
API Handler (claude/openai/gemini)
  │  解析 body → 提取 model
  │
  ▼
service::relay::forward()
  │
  ├─► scheduler::select_account()
  │     ├─ session::get_sticky() → Redis 查粘性会话
  │     ├─ 查询 accounts 表 (platform + status = active)
  │     ├─ pick_least_loaded() → Redis SCARD 查并发数
  │     └─ session::set_sticky() → Redis 绑定
  │
  ├─► FingerprintEngine::build_client()
  │     ├─ TLS profile → rustls 配置
  │     ├─ HTTP/2 SETTINGS → hyper 参数
  │     ├─ Proxy → reqwest::Proxy
  │     └─ 返回 reqwest::Client
  │
  ├─► headers::apply_header_profile()
  │     ├─ 按 profile.header_order 排序
  │     ├─ 注入 extra_headers
  │     ├─ 设置 User-Agent (模板渲染)
  │     └─ sanitize_outgoing() 清除代理头
  │
  ├─► http_client.post(upstream_url).send()
  │
  └─► 流式响应 → SSE → 返回客户端
        │
        └─► cost::record_usage() (异步)
```

---

## 指纹引擎详解

### 指纹维度

| 维度 | 文件 | 说明 |
|------|------|------|
| **TLS ClientHello** | `fingerprint/tls.rs` | 密码套件顺序、TLS 扩展、椭圆曲线、ALPN |
| **HTTP/2 SETTINGS** | `fingerprint/profile.rs` | HEADER_TABLE_SIZE, INITIAL_WINDOW_SIZE 等 |
| **Header 排序** | `fingerprint/headers.rs` | HTTP 头部发送顺序 (不同客户端顺序不同) |
| **User-Agent** | `fingerprint/headers.rs` | 可模板化、可轮换版本号 |
| **Extra Headers** | `fingerprint/profile.rs` | x-stainless-* 等客户端特征头 |

### 内置 Profile

| Profile | 用途 | TLS 特征 |
|---------|------|----------|
| `chrome-131-macos` | 默认 — 伪装 Chrome 浏览器 | 标准 Chrome cipher order, h2 ALPN |
| `claude-cli` | 伪装 Claude Code CLI | Node.js TLS stack, x-stainless-* headers |
| `node20-axios` | 伪装 Node.js 应用 | OpenSSL cipher order, HTTP/1.1 only |

### 自定义 Profile

通过 Admin API 创建：

```json
POST /admin/fingerprints
{
  "name": "custom-safari-18",
  "tls_profile": {
    "cipher_suites": ["TLS_AES_128_GCM_SHA256", "TLS_AES_256_GCM_SHA384"],
    "extensions": ["server_name", "supported_versions"],
    "curves": ["X25519", "P-256"],
    "alpn": ["h2", "http/1.1"]
  },
  "http2_settings": {
    "header_table_size": 4096,
    "initial_window_size": 2097152
  },
  "header_order": [":method", ":scheme", ":path", ":authority", "accept"],
  "user_agent_template": "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) ... Safari/605.1.15",
  "extra_headers": {}
}
```

每个 Account 可绑定一个 `fingerprint_profile_id`。

---

## 数据模型

### PostgreSQL 表

| 表 | 说明 |
|----|------|
| `accounts` | 上游 AI 账户 (OAuth token / API Key, AES 加密) |
| `api_keys` | 客户端认证密钥 (SHA-256 存储) |
| `fingerprint_profiles` | 自定义指纹配置 (JSONB) |
| `usage_logs` | 请求用量日志 (token 数 + 成本) |
| `admin_sessions` | 管理员会话 |

### Redis 键空间

| 键模式 | 说明 | TTL |
|--------|------|-----|
| `rl:{key_id}:{minute}` | 速率限制计数 | 60s |
| `sess:{hash}` | 粘性会话绑定 | 可配置 |
| `conc:{account_id}` | 账户并发 Sorted Set | 自动清理 |
| `cooldown:{account_id}` | 过载冷却标记 | 可配置 |

---

## 安全设计

1. **敏感数据加密存储** — OAuth token 使用 AES-256-GCM 加密，每次不同 nonce
2. **API Key 哈希存储** — 仅存 SHA-256，客户端密钥仅在创建时返回一次
3. **日志脱敏** — `util/mask.rs` 自动 REDACT token/password/secret 字段
4. **请求头清洗** — `headers::sanitize_outgoing()` 去除所有代理/CDN 头
5. **错误不泄露** — Internal Error 只返回 "internal server error"，详情仅写日志
6. **零项目标识** — 响应头、错误消息、日志中无项目名/仓库地址/版本指纹

---

## 反指纹设计原则

### 已移除的指纹 (对比 CRS/S2A)

| 指纹类别 | CRS 中的值 | S2A 中的值 | nexus-gate |
|----------|-----------|-----------|------------|
| API Key 前缀 | `cr_` | 无 | `nk-` (可配置) |
| 自定义 Header | `x-cr-api-key` | 无 | 无 |
| User-Agent | `claude-relay-service/*` | 无 | Profile 控制 |
| 管理面板路径 | `/admin-next/` | `/admin/` | `/admin/` (可配置) |
| 错误码 | E001-E015 | code/message | 标准格式 |
| 日志文件名 | `claude-relay-*.log` | `sub2api` | stdout/文件 (无品牌) |
| 加密盐 | `claude-relay-salt` | 无 | SHA-256(key) 派生 |
| Webhook 标识 | `service: claude-relay-service` | 无 | 无硬编码标识 |
| GitHub 引用 | `Wei-Shaw/claude-relay-service` | `Wei-Shaw/sub2api` | 无 |
| Redis Key 前缀 | `apikey:`, `sticky_session:` | `openai:account:` | `rl:`, `sess:`, `conc:` |
| 健康检查 | 自定义格式 | `{"status":"ok"}` | `{"status":"ok","version":"x"}` |
| 数据库表名 | N/A (Redis) | Ent generated | 自定义简洁命名 |

### 运行时反检测

1. **TLS 指纹旋转** — 每个账户可绑定不同的 TLS profile
2. **出口 IP 隔离** — 每个账户可配置独立代理
3. **Header 顺序模拟** — 按目标客户端的真实 Header 排序
4. **版本号跟随** — User-Agent 模板支持动态版本轮换
5. **请求频率控制** — 速率限制确保单账户请求频率符合合理范围

---

## 开发工作流

```bash
# 首次设置
cp .env.example .env          # 编辑 .env 填入真实密钥
docker compose up -d postgres redis  # 启动依赖
make migrate                   # 运行数据库迁移

# 日常开发
make dev                       # cargo watch 热重载
make test                      # 运行所有测试
make lint                      # clippy 检查
make fmt                       # rustfmt 格式化

# 生产构建
make build                     # release 编译 (含 LTO)
make docker                    # Docker 镜像
```

---

## 模块测试策略

每个模块包含 `#[cfg(test)] mod tests` 内嵌单测：

| 模块 | 测试内容 |
|------|----------|
| `config` | TOML 解析、默认值 |
| `error` | 状态码映射、错误类型字符串、内部错误不泄露 |
| `middleware/auth` | Key 提取 (Bearer/header/query)、SHA-256 hash |
| `middleware/rate_limit` | 分钟窗口计算 |
| `service/apikey` | Key 生成格式、唯一性、格式校验 |
| `service/session` | Hash 确定性、不同输入不同 hash |
| `service/cost` | 成本计算精度、默认定价 |
| `fingerprint/profile` | Profile 序列化回环、ALPN 配置 |
| `fingerprint/tls` | JA3 确定性、不同 profile 不同 JA3 |
| `fingerprint/headers` | Header 排序、extra header 注入、代理头清除 |
| `fingerprint/engine` | Client 构建 (默认/代理/自定义 profile) |
| `model/*` | 反序列化 |
| `util/crypto` | 加密解密回环、错误 key 解密失败 |
| `util/mask` | 脱敏覆盖、JSON 递归去敏 |
| `util/proxy` | URL 格式校验 |
| `util/stream` | SSE 事件格式 |

---

## 路线图

- [ ] Phase 1: 脚手架 + 核心中转 (Claude/OpenAI/Gemini)
- [ ] Phase 2: 完整指纹引擎 (boring-ssl 集成 for 真实 TLS 控制)
- [ ] Phase 3: Admin SPA 前端 (React/Svelte, 非 Vue+Element Plus)
- [ ] Phase 4: Bedrock / Azure 支持
- [ ] Phase 5: WebSocket 中转 (OpenAI Responses API)
- [ ] Phase 6: OAuth token 自动刷新
- [ ] Phase 7: 高级调度 (权重轮询、健康检查、自动恢复)
