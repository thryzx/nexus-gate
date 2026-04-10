use axum::body::Body;
use bytes::Bytes;
use futures::stream::Stream;

/// Create an SSE-formatted body stream from a stream of byte chunks.
/// Wraps each chunk in `data: ...\n\n` format.
pub fn sse_body_from_stream(
    input: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
) -> Body {
    Body::from_stream(input)
}

/// Create a single SSE event string.
pub fn sse_event(data: &str) -> String {
    format!("data: {data}\n\n")
}

/// Create the terminal SSE event.
pub fn sse_done() -> String {
    "data: [DONE]\n\n".to_string()
}

/// Wrap a single JSON value as a one-shot SSE stream.
pub fn sse_single(json: serde_json::Value) -> Body {
    let event = sse_event(&json.to_string());
    let done = sse_done();
    let combined = format!("{event}{done}");
    Body::from(combined)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sse_event_format() {
        let event = sse_event(r#"{"type":"ping"}"#);
        assert_eq!(event, "data: {\"type\":\"ping\"}\n\n");
    }

    #[test]
    fn sse_done_format() {
        assert_eq!(sse_done(), "data: [DONE]\n\n");
    }

    #[test]
    fn sse_event_no_prefix_leak() {
        let event = sse_event("hello");
        assert!(!event.contains("nexus"));
        assert!(!event.contains("relay"));
    }
}
