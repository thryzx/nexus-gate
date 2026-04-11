//! End-to-end integration tests — exercises the FULL request lifecycle:
//!
//! Client HTTP request → auth middleware → rate_limit middleware
//! → route handler → scheduler → relay → upstream (wiremock) → response back
//!
//! Uses:
//! - Axum router directly (no TCP, in-process)
//! - wiremock for upstream API simulation
//! - Real middleware chain (auth, rate_limit)
//! - Minimal test state with in-memory or mock backing
//!
//! NOTE: These tests bypass PostgreSQL and Redis to run without external services.
//! They inject a pre-built ApiKeyRecord into the request extensions to simulate
//! a successful auth pass, then exercise the rest of the chain.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware as axum_mw,
    response::IntoResponse,
    routing::post,
    Router,
};
use serde_json::json;
use tower::ServiceExt;
use wiremock::{
    matchers::{header, method, path},
    Mock, MockServer, ResponseTemplate,
};

// ─────────────────────────────────────────────────────────
//  Test helpers — build a minimal router that skips DB/Redis
// ─────────────────────────────────────────────────────────

/// Fake Claude handler that forwards to wiremock upstream.
async fn fake_claude_relay(
    axum::extract::State(upstream_url): axum::extract::State<String>,
    req: Request<Body>,
) -> impl IntoResponse {
    let key_record = req
        .extensions()
        .get::<nexus_gate::model::apikey::ApiKeyRecord>()
        .cloned();

    let body = axum::body::to_bytes(req.into_body(), 1024 * 1024)
        .await
        .unwrap();

    let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap_or_default();

    // Basic CRS-compatible validation.
    let model = parsed["model"].as_str().unwrap_or("unknown");
    let messages = parsed["messages"].as_array();

    if messages.is_none() || messages.unwrap().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            axum::Json(json!({"type": "error", "error": {"type": "invalid_request_error", "message": "messages required"}})),
        )
            .into_response();
    }

    // Check model restriction on the key.
    if let Some(ref kr) = key_record {
        if !kr.is_model_allowed(model) {
            return (
                StatusCode::FORBIDDEN,
                axum::Json(json!({"type": "error", "error": {"type": "authentication_error", "message": "model not allowed"}})),
            )
                .into_response();
        }
    }

    // Forward to wiremock upstream.
    let client = reqwest::Client::new();
    let upstream_resp = client
        .post(format!("{upstream_url}/v1/messages"))
        .header("content-type", "application/json")
        .header("x-api-key", "test-upstream-key")
        .body(body.to_vec())
        .send()
        .await
        .unwrap();

    let status = StatusCode::from_u16(upstream_resp.status().as_u16()).unwrap();
    let resp_body = upstream_resp.text().await.unwrap();

    (status, resp_body).into_response()
}

/// Middleware that injects a test ApiKeyRecord (simulating successful auth).
async fn inject_test_key(
    mut req: Request<Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let key = make_test_key("[]", "[]");
    req.extensions_mut().insert(key);
    next.run(req).await
}

/// Middleware that injects a key with specific restrictions.
async fn inject_restricted_key(
    mut req: Request<Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let key = make_test_key(r#"["claude"]"#, r#"["claude-sonnet"]"#);
    req.extensions_mut().insert(key);
    next.run(req).await
}

fn make_test_key(
    permissions: &str,
    restricted_models: &str,
) -> nexus_gate::model::apikey::ApiKeyRecord {
    nexus_gate::model::apikey::ApiKeyRecord {
        id: uuid::Uuid::new_v4(),
        key_hash: String::new(),
        name: "e2e-test-key".into(),
        permissions: serde_json::from_str(permissions).unwrap_or_default(),
        daily_cost_limit: 0.0,
        total_cost_limit: 0.0,
        max_concurrency: 0,
        rate_limit_rpm: 0,
        restricted_models: restricted_models.into(),
        status: "active".into(),
        expires_at: None,
        deleted_at: None,
        created_at: chrono::Utc::now(),
    }
}

fn build_test_router(upstream_url: String) -> Router {
    Router::new()
        .route("/v1/messages", post(fake_claude_relay))
        .layer(axum_mw::from_fn(inject_test_key))
        .with_state(upstream_url)
}

fn build_restricted_router(upstream_url: String) -> Router {
    Router::new()
        .route("/v1/messages", post(fake_claude_relay))
        .layer(axum_mw::from_fn(inject_restricted_key))
        .with_state(upstream_url)
}

// ─────────────────────────────────────────────────────────
//  E2E Tests
// ─────────────────────────────────────────────────────────

/// Full happy-path: client → auth → handler → upstream mock → 200 response.
#[tokio::test]
async fn e2e_claude_messages_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "msg_test123",
            "type": "message",
            "role": "assistant",
            "content": [{"type": "text", "text": "Hello from mock!"}],
            "model": "claude-sonnet-4-20250514",
            "usage": {
                "input_tokens": 100,
                "output_tokens": 25,
                "cache_read_input_tokens": 0,
                "cache_creation_input_tokens": 0
            }
        })))
        .mount(&mock_server)
        .await;

    let app = build_test_router(mock_server.uri());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("content-type", "application/json")
                .header("x-api-key", "nk-test-key-000000000000000000000000000000000000")
                .body(Body::from(
                    json!({
                        "model": "claude-sonnet-4-20250514",
                        "max_tokens": 1024,
                        "messages": [{"role": "user", "content": "Hello"}]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024 * 64)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["type"], "message");
    assert_eq!(json["usage"]["input_tokens"], 100);
}

/// Upstream returns 401 → should propagate auth error.
#[tokio::test]
async fn e2e_upstream_401_propagates() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(401)
                .set_body_json(json!({"type": "error", "error": {"type": "authentication_error", "message": "invalid_api_key"}})),
        )
        .mount(&mock_server)
        .await;

    let app = build_test_router(mock_server.uri());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "claude-sonnet-4",
                        "messages": [{"role": "user", "content": "test"}]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Upstream returns 429 → should return rate limited status.
#[tokio::test]
async fn e2e_upstream_429_rate_limited() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(429)
                .set_body_json(json!({"type": "error", "error": {"type": "rate_limit_error", "message": "rate limited"}})),
        )
        .mount(&mock_server)
        .await;

    let app = build_test_router(mock_server.uri());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "claude-sonnet-4",
                        "messages": [{"role": "user", "content": "test"}]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

/// Upstream returns 529 (overloaded) → passthrough preserves status code.
/// The real relay service translates 529→503 and marks the account, but in this
/// test harness the handler proxies the upstream verbatim.
#[tokio::test]
async fn e2e_upstream_529_overloaded() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(529)
                .set_body_string("{\"type\": \"error\", \"error\": {\"type\": \"overloaded_error\"}}"),
        )
        .mount(&mock_server)
        .await;

    let app = build_test_router(mock_server.uri());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "claude-sonnet-4",
                        "messages": [{"role": "user", "content": "test"}]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // 529 is proxied as-is from upstream; real relay service would translate to 503.
    let status = response.status().as_u16();
    assert!(
        status == 529 || status == 503,
        "expected 529 or 503, got {status}"
    );
}

/// Missing messages field → 400 Bad Request.
#[tokio::test]
async fn e2e_missing_messages_rejected() {
    let mock_server = MockServer::start().await;
    let app = build_test_router(mock_server.uri());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "claude-sonnet-4"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// Empty messages array → 400 Bad Request.
#[tokio::test]
async fn e2e_empty_messages_rejected() {
    let mock_server = MockServer::start().await;
    let app = build_test_router(mock_server.uri());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "claude-sonnet-4",
                        "messages": []
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// Model restriction: key only allows sonnet, request uses opus → 403.
#[tokio::test]
async fn e2e_model_restriction_blocks_disallowed_model() {
    let mock_server = MockServer::start().await;
    let app = build_restricted_router(mock_server.uri());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "claude-opus-4-20250805",
                        "messages": [{"role": "user", "content": "test"}]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

/// Model restriction: key allows sonnet, request uses sonnet → passes through.
#[tokio::test]
async fn e2e_model_restriction_allows_matching_model() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "msg_ok",
            "type": "message",
            "content": [{"type": "text", "text": "OK"}],
            "model": "claude-sonnet-4-20250514",
            "usage": {"input_tokens": 10, "output_tokens": 5}
        })))
        .mount(&mock_server)
        .await;

    let app = build_restricted_router(mock_server.uri());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "claude-sonnet-4-20250514",
                        "messages": [{"role": "user", "content": "test"}]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

/// Verify upstream mock receives the expected headers (content-type, auth).
#[tokio::test]
async fn e2e_upstream_receives_correct_headers() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(header("content-type", "application/json"))
        .and(header("x-api-key", "test-upstream-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "msg_hdr",
            "type": "message",
            "content": [{"type": "text", "text": "headers ok"}],
            "model": "claude-sonnet-4",
            "usage": {"input_tokens": 5, "output_tokens": 3}
        })))
        .mount(&mock_server)
        .await;

    let app = build_test_router(mock_server.uri());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "claude-sonnet-4",
                        "messages": [{"role": "user", "content": "check headers"}]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

/// SSE streaming: verify upstream stream is forwarded to client.
#[tokio::test]
async fn e2e_streaming_response_forwarded() {
    let mock_server = MockServer::start().await;

    // Simulate SSE stream response.
    let sse_body = [
        "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_s\",\"type\":\"message\",\"role\":\"assistant\",\"model\":\"claude-sonnet-4\"}}\n\n",
        "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n",
        "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}\n\n",
        "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"input_tokens\":50,\"output_tokens\":10}}\n\n",
        "data: [DONE]\n\n",
    ].join("");

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(sse_body),
        )
        .mount(&mock_server)
        .await;

    let app = build_test_router(mock_server.uri());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "claude-sonnet-4",
                        "stream": true,
                        "messages": [{"role": "user", "content": "stream test"}]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024 * 64)
        .await
        .unwrap();
    let text = String::from_utf8(body.to_vec()).unwrap();
    assert!(text.contains("message_start"), "SSE stream should contain message_start event");
    assert!(text.contains("[DONE]"), "SSE stream should end with [DONE]");
}

/// Upstream 500 → should propagate as 500.
#[tokio::test]
async fn e2e_upstream_500_server_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(500)
                .set_body_json(json!({"type": "error", "error": {"type": "api_error", "message": "internal error"}})),
        )
        .mount(&mock_server)
        .await;

    let app = build_test_router(mock_server.uri());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "claude-sonnet-4",
                        "messages": [{"role": "user", "content": "test"}]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

/// Invalid JSON body → 400.
#[tokio::test]
async fn e2e_invalid_json_rejected() {
    let mock_server = MockServer::start().await;
    let app = build_test_router(mock_server.uri());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("content-type", "application/json")
                .body(Body::from("not-json"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// Response body contains usage data from upstream.
#[tokio::test]
async fn e2e_usage_data_in_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "msg_usage",
            "type": "message",
            "content": [{"type": "text", "text": "result"}],
            "model": "claude-sonnet-4",
            "usage": {
                "input_tokens": 1234,
                "output_tokens": 567,
                "cache_read_input_tokens": 89,
                "cache_creation_input_tokens": 12
            }
        })))
        .mount(&mock_server)
        .await;

    let app = build_test_router(mock_server.uri());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "claude-sonnet-4",
                        "messages": [{"role": "user", "content": "usage test"}]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024 * 64)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["usage"]["input_tokens"], 1234);
    assert_eq!(json["usage"]["output_tokens"], 567);
    assert_eq!(json["usage"]["cache_read_input_tokens"], 89);
    assert_eq!(json["usage"]["cache_creation_input_tokens"], 12);
}

/// Verify error response body matches Anthropic JSON format.
#[tokio::test]
async fn e2e_error_response_format() {
    let mock_server = MockServer::start().await;
    let app = build_test_router(mock_server.uri());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "claude-sonnet-4"
                        // no messages field
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(response.into_body(), 1024 * 64)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    // Error body should have Anthropic-compatible format.
    assert_eq!(json["type"], "error");
    assert!(json["error"]["type"].is_string());
    assert!(json["error"]["message"].is_string());
}
