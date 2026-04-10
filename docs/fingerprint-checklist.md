# nexus-gate 指纹清单

本文记录从 claude-relay-service (CRS) 和 sub2api (S2A) 中识别的所有可追踪指纹，
以及 nexus-gate 对每项指纹的处理方式。

## 状态图例

- ✅ 已完全去除
- 🔄 已替换为自有方案
- 📋 待后续实现

---

## CRS 指纹对照

| # | 指纹 | 原始值 | nexus-gate 处理 | 状态 |
|---|------|--------|-----------------|------|
| 1 | API Key 前缀 | `cr_` | `nk-` (可配置) | 🔄 |
| 2 | User-Agent (上游) | `claude-relay-service/1.0.0` | FingerprintProfile 控制 | 🔄 |
| 3 | package.json name | `claude-relay-service` | `nexus-gate` (Cargo.toml) | 🔄 |
| 4 | 自定义 Header | `x-cr-api-key` | 不存在 | ✅ |
| 5 | 响应 Header 泄露 | `X-Request-ID` + CORS expose | `x-req-id` (无 CORS 暴露) | 🔄 |
| 6 | Admin 路径 | `/admin-next/` | `/admin/` | 🔄 |
| 7 | 错误码体系 | `E001`-`E015` | 标准 Anthropic 格式 | 🔄 |
| 8 | 日志文件名 | `claude-relay-*.log` | stdout / tracing | ✅ |
| 9 | 加密盐 | `claude-relay-salt` | SHA-256(key) 自动派生 | ✅ |
| 10 | Webhook payload | `service: 'claude-relay-service'` | 无 webhook (或自定义) | ✅ |
| 11 | GitHub 仓库 | `Wei-Shaw/claude-relay-service` | 无引用 | ✅ |
| 12 | Redis Key | `apikey:*`, `sticky_session:*` | `rl:*`, `sess:*`, `conc:*` | 🔄 |
| 13 | 日志 emoji | `📥 🔄 🎯 ⏳ 🔌` | 无 emoji, 结构化 JSON log | ✅ |
| 14 | 默认端口 | `3000` | `7900` | 🔄 |
| 15 | Stainless 头 | 固定版本 `0.55.1` | Profile 可配置, 版本可轮换 | 🔄 |
| 16 | Claude CLI UA | `claude-cli/1.0.57 (external, cli)` | 模板化 `{version}` 动态替换 | 🔄 |
| 17 | Warmup 检测 | 固定文本匹配 | 不存在 | ✅ |
| 18 | Anthropic Beta 头 | 固定字符串拼接 | 由 Profile 控制 | 🔄 |

---

## S2A 指纹对照

| # | 指纹 | 原始值 | nexus-gate 处理 | 状态 |
|---|------|--------|-----------------|------|
| 1 | Go Module | `github.com/Wei-Shaw/sub2api` | Rust crate `nexus-gate` | ✅ |
| 2 | Docker 标签 | `Sub2API - AI API Gateway` | 无品牌标签 | ✅ |
| 3 | 二进制名 | `/app/sub2api` | `/usr/local/bin/nexus-gate` | 🔄 |
| 4 | 启动输出 | `Sub2API {version}` | `starting nexus-gate` | 🔄 |
| 5 | 日志 service_name | `sub2api` | `nexus_gate` (tracing) | 🔄 |
| 6 | 数据库表名 | Ent 生成的表名 | 自定义简洁表名 | ✅ |
| 7 | 容器用户 | `sub2api:sub2api` | Alpine 默认 | ✅ |
| 8 | Setup 端点 | `/setup/status` | 不存在 | ✅ |
| 9 | 前端 API 路径 | `/api/v1/admin/*` | `/admin/*` | 🔄 |
| 10 | TLS profile 名 | `claude_cli_v2` | 自定义命名 | ✅ |
| 11 | 域名引用 | `sub2api.org`, `pincc.ai` | 无引用 | ✅ |
| 12 | OPS 错误分类 | phase/owner/source/severity | 不存在 | ✅ |
| 13 | 默认端口 | `8080` | `7900` | 🔄 |
| 14 | 响应格式 | `{"code": 0, "data": ...}` | 标准 API 格式 | 🔄 |

---

## 行为指纹 (上游可检测)

| # | 行为 | 风险 | nexus-gate 缓解措施 |
|---|------|------|---------------------|
| 1 | 多账户同 IP 轮换 | 🔴 高 | 每账户独立 proxy_url | 📋 |
| 2 | TLS ClientHello 固定 | 🔴 高 | FingerprintProfile 轮换 | 🔄 |
| 3 | 固定 anthropic-beta 头 | 🟡 中 | Profile extra_headers 控制 | 🔄 |
| 4 | 批量 token 刷新 | 🟡 中 | 待实现: 随机化刷新间隔 | 📋 |
| 5 | 高频同 IP 请求 | 🟡 中 | rate_limit 限流 | ✅ |
| 6 | 请求排队延迟特征 | 🟢 低 | 正常排队行为 | ✅ |

---

## 检查清单 (新增功能时)

每次新增功能，必须检查以下项：

- [ ] 响应头中没有项目名/版本/仓库信息
- [ ] 错误消息中没有内部细节
- [ ] 日志中敏感数据已脱敏
- [ ] Redis key 使用简短前缀 (非描述性)
- [ ] 上游请求 Header 由 FingerprintProfile 控制
- [ ] 无硬编码 User-Agent / 版本号
- [ ] 新依赖不会在 HTTP 头中泄露标识
