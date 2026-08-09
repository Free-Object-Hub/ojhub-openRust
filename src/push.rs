use axum::{Json, extract::State};
use serde::Deserialize;
use std::io::Cursor;
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use p256::SecretKey;
use pkcs8::EncodePrivateKey;
use web_push::{
    SubscriptionInfo, VapidSignatureBuilder, WebPushMessageBuilder,
    ContentEncoding, WebPushError, IsahcWebPushClient, WebPushClient,
};
//use sqlx::MySqlPool;
use crate::AppState;

#[derive(Deserialize)]
pub struct SendPushRequest {
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
    pub title: String,
    pub body: String,
}

fn vapid_key_to_pem(raw_base64url: &str) -> Result<String, String> {
    let raw_bytes = URL_SAFE_NO_PAD.decode(raw_base64url)
        .map_err(|e| format!("base64 decode error: {e}"))?;

    let secret_key = SecretKey::from_slice(&raw_bytes)
        .map_err(|e| format!("invalid EC key: {e}"))?;

    let pem = secret_key.to_pkcs8_pem(Default::default())
        .map_err(|e| format!("pem encode error: {e}"))?;

    Ok(pem.to_string())
}

pub async fn send_push_handler(
    State(_pool): State<AppState>, // пока не используется, но роутер требует State везде
    Json(req): Json<SendPushRequest>,
) -> String {
    let subscription_info = SubscriptionInfo::new(&req.endpoint, &req.p256dh, &req.auth);

    let vapid_raw = match std::env::var("VAPID_PRIVATE_KEY") {
        Ok(k) => k,
        Err(_) => {
            eprintln!("VAPID_PRIVATE_KEY missing");
            return "-1".to_string();
        }
    };

    let vapid_pem = match vapid_key_to_pem(&vapid_raw) {
        Ok(pem) => pem,
        Err(e) => {
            eprintln!("vapid key conversion error: {e}");
            return "-6".to_string();
        }
    };

    let sig_builder = match VapidSignatureBuilder::from_pem(
        Cursor::new(vapid_pem.as_bytes()),
        &subscription_info,
    ) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("vapid builder error: {e}");
            return "-1".to_string();
        }
    };

    let payload = format!(r#"{{"title":"{}","body":"{}"}}"#, req.title, req.body);

    let mut message_builder = WebPushMessageBuilder::new(&subscription_info);
    message_builder.set_payload(ContentEncoding::Aes128Gcm, payload.as_bytes());

    let vapid_signature = match sig_builder.build() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("vapid signature build error: {e}");
            return "-2".to_string();
        }
    };
    message_builder.set_vapid_signature(vapid_signature);

    let message = match message_builder.build() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("message build error: {e}");
            return "-3".to_string();
        }
    };

    let client = match IsahcWebPushClient::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("client init error: {e}");
            return "-4".to_string();
        }
    };

    match client.send(message).await {
        Ok(_) => "1".to_string(),
        Err(WebPushError::EndpointNotValid) | Err(WebPushError::EndpointNotFound) => "410".to_string(),
        Err(e) => {
            eprintln!("send error: {e}");
            "-5".to_string()
        }
    }
}
