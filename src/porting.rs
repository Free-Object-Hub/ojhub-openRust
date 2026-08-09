use axum::http::HeaderMap;

pub fn extract_ip(headers: &HeaderMap) -> String {
    headers
        .get("X-Real-Ip")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
}
