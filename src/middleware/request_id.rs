use axum::{body::Body, http::Request, middleware::Next, response::Response};
use uuid::Uuid;

const REQUEST_ID_HEADER: &str = "x-req-id";

/// Generate a unique request ID and attach it to both the request
/// extensions and the response header.
pub async fn inject(mut req: Request<Body>, next: Next) -> Response {
    let id = Uuid::new_v4().to_string();
    req.extensions_mut().insert(RequestId(id.clone()));

    let mut resp = next.run(req).await;
    resp.headers_mut().insert(
        REQUEST_ID_HEADER,
        id.parse().unwrap_or_else(|_| "unknown".parse().unwrap()),
    );

    resp
}

#[derive(Clone, Debug)]
pub struct RequestId(pub String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_id_is_uuid_format() {
        let id = Uuid::new_v4().to_string();
        assert_eq!(id.len(), 36);
        assert!(id.contains('-'));
    }
}
