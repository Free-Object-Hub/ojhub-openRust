pub mod middleware;

use sqlx::FromRow;
use maxminddb::geoip2;
use std::net::IpAddr;
use std::str::FromStr;
use sqlx::MySqlPool;

pub fn exploit_patch(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

pub fn get_city(reader: &maxminddb::Reader<Vec<u8>>, ip: &str) -> (String, String) {
    let unknown = ("Unknown".to_string(), "Unknown".to_string());
    let addr = match IpAddr::from_str(ip) {
        Ok(a) => a,
        Err(_) => return unknown,
    };
    let city: geoip2::City = match reader.lookup(addr) {
        Ok(Some(c)) => c,
        _ => return unknown,
    };
    let country = city
        .country
        .as_ref()
        .and_then(|c| c.names.as_ref())
        .and_then(|n| n.get("en"))
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let city_name = city
        .city
        .as_ref()
        .and_then(|c| c.names.as_ref())
        .and_then(|n| n.get("en"))
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    (country, city_name)
}

pub async fn recaptcha_verify(token: &str) -> Result<bool, String> {
    return Ok(true); // заглушка, как в openGo
    #[allow(unreachable_code)]
    {
        let secret = std::env::var("RECAPTCHA").map_err(|_| "RECAPTCHA env missing".to_string())?;
        let client = reqwest::Client::new();
        let resp = client
            .post("https://www.google.com/recaptcha/api/siteverify")
            .form(&[("secret", secret.as_str()), ("response", token)])
            .send()
            .await
            .map_err(|e| format!("recaptcha request failed: {e}"))?;
        #[derive(serde::Deserialize)]
        struct RecaptchaResponse {
            success: bool,
        }
        let result: RecaptchaResponse = resp
            .json()
            .await
            .map_err(|e| format!("recaptcha parse failed: {e}"))?;
        Ok(result.success)
    }
}
