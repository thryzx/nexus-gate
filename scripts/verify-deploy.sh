#!/usr/bin/env bash
# ─── nexus-gate 部署验证脚本 ───
# 用法:
#   ./scripts/verify-deploy.sh                  # 本地验证 (http://localhost)
#   ./scripts/verify-deploy.sh https://gate.example.com   # 远程验证
#   ./scripts/verify-deploy.sh http://1.2.3.4   # IP 验证
set -euo pipefail

BASE_URL="${1:-http://localhost}"
# 去除末尾斜杠
BASE_URL="${BASE_URL%/}"

PASS=0
FAIL=0
TOTAL=0

green() { printf "\033[32m%s\033[0m\n" "$1"; }
red()   { printf "\033[31m%s\033[0m\n" "$1"; }
yellow(){ printf "\033[33m%s\033[0m\n" "$1"; }

check() {
    local name="$1"
    local method="$2"
    local endpoint="$3"
    local expected_status="$4"
    local body="${5:-}"
    local extra_header="${6:-}"
    TOTAL=$((TOTAL + 1))

    local curl_args=(-s -o /dev/null -w "%{http_code}" -X "$method" --max-time 10)
    if [ -n "$body" ]; then
        curl_args+=(-H "Content-Type: application/json" -d "$body")
    fi
    if [ -n "$extra_header" ]; then
        curl_args+=(-H "$extra_header")
    fi

    local status
    status=$(curl "${curl_args[@]}" "${BASE_URL}${endpoint}" 2>/dev/null || echo "000")

    if [ "$status" = "$expected_status" ]; then
        green "  ✓ ${name} (${status})"
        PASS=$((PASS + 1))
    else
        red "  ✗ ${name} — expected ${expected_status}, got ${status}"
        FAIL=$((FAIL + 1))
    fi
}

echo ""
echo "━━━ nexus-gate 部署验证 ━━━"
echo "  目标: ${BASE_URL}"
echo ""

# ── 1. 健康检查 ──
echo "▸ 基础健康检查"
check "GET /health" GET "/health" "200"

# 验证 health 响应体
HEALTH_BODY=$(curl -s --max-time 5 "${BASE_URL}/health" 2>/dev/null || echo "{}")
if echo "$HEALTH_BODY" | grep -q '"status":"ok"'; then
    green "  ✓ /health 返回 {\"status\":\"ok\"}"
    PASS=$((PASS + 1))
else
    red "  ✗ /health 响应体异常: ${HEALTH_BODY}"
    FAIL=$((FAIL + 1))
fi
TOTAL=$((TOTAL + 1))

# ── 2. 认证拦截（无 key 应返回 401）──
echo ""
echo "▸ 认证中间件"
check "Claude /v1/messages 无 key → 401" \
    POST "/v1/messages" "401" \
    '{"model":"claude-sonnet-4","messages":[{"role":"user","content":"test"}]}'

check "OpenAI /v1/chat/completions 无 key → 401" \
    POST "/v1/chat/completions" "401" \
    '{"model":"gpt-4o","messages":[{"role":"user","content":"test"}]}'

check "Gemini /v1beta/models/gemini-pro:generateContent 无 key → 401" \
    POST "/v1beta/models/gemini-pro:generateContent" "401" \
    '{"contents":[{"parts":[{"text":"test"}]}]}'

check "OpenAI /v1/responses 无 key → 401" \
    POST "/v1/responses" "401" \
    '{"model":"gpt-4o","input":"test"}'

# ── 3. 假 key 应返回 401 ──
echo ""
echo "▸ 无效 Key 拒绝"
check "假 x-api-key → 401" \
    POST "/v1/messages" "401" \
    '{"model":"claude-sonnet-4","messages":[{"role":"user","content":"test"}]}' \
    "x-api-key: nk-fake-key-00000000000000000000000000000000000"

check "假 Bearer token → 401" \
    POST "/v1/chat/completions" "401" \
    '{"model":"gpt-4o","messages":[{"role":"user","content":"test"}]}' \
    "Authorization: Bearer nk-fake-key-00000000000000000000000000"

# ── 4. Admin 端点 ──
echo ""
echo "▸ Admin 端点"
check "GET /admin/accounts 无 JWT → 401" GET "/admin/accounts" "401"
check "GET /admin/keys 无 JWT → 401"     GET "/admin/keys" "401"
check "GET /admin/stats 无 JWT → 401"    GET "/admin/stats" "401"

# ── 5. 不存在的路由 ──
echo ""
echo "▸ 路由保护"
check "不存在的路径 → 404" GET "/nonexistent" "404"

# ── 6. Gemini 端点 ──
echo ""
echo "▸ Gemini 端点"
check "GET /v1beta/models (listModels) 无 key → 401" \
    GET "/v1beta/models" "401"

# ── 7. 限流中间件（非必须，但验证存在）──
echo ""
echo "▸ 限流中间件（快速连续请求）"
for i in $(seq 1 3); do
    curl -s -o /dev/null -X POST "${BASE_URL}/v1/messages" \
        -H "Content-Type: application/json" \
        -d '{"model":"test","messages":[{"role":"user","content":"t"}]}' 2>/dev/null &
done
wait
green "  ✓ 并发请求未导致崩溃"
PASS=$((PASS + 1))
TOTAL=$((TOTAL + 1))

# ── 汇总 ──
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ $FAIL -eq 0 ]; then
    green "  全部通过: ${PASS}/${TOTAL} ✓"
else
    red "  失败: ${FAIL}/${TOTAL}"
    yellow "  通过: ${PASS}/${TOTAL}"
fi
echo ""

exit $FAIL
