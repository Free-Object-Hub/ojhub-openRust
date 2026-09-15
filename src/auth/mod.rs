pub mod middleware;
pub mod handlers;

use sqlx::FromRow;
use sqlx::MySqlPool;

pub fn exploit_patch(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
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
