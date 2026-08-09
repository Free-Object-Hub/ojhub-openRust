use axum::{extract::{Form, State}, http::{HeaderMap, StatusCode}, response::IntoResponse};
use std::collections::HashMap;
use crate::auth::{recaptcha_verify, exploit_patch, get_user_by_email, get_user_by_username, get_city};
use crate::devices::insert_device;
use crate::porting::extract_ip;

#[derive(serde::Serialize)]
pub struct LoginResponse {
    #[serde(rename = "ID")]
    pub id: i32,
    pub username: String,
    #[serde(rename = "isActive")]
    pub is_active: i32,
    pub role: i32,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub token: String,
    pub resume: String,
    pub socials: String,
    #[serde(rename = "cityData")]
    pub city_data: [String; 2],
}

pub async fn login_handler(
    State(state): State<crate::AppState>,
    headers: HeaderMap,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    /* HELP: временно, убрать на проде
    let recaptcha_token = form.get("g-recaptcha-response").cloned().unwrap_or_default();
    if recaptcha_token.is_empty() {
        return (StatusCode::OK, "-3".to_string());
    }
    match recaptcha_verify(&recaptcha_token).await {
        Ok(true) => {}
        _ => return (StatusCode::OK, "-3".to_string()),
    }
    */

    let raw_username = form.get("username").cloned().unwrap_or_default();
    let username = exploit_patch(&raw_username);
    let password = form.get("password").cloned().unwrap_or_default();

    let user_result = if username.contains('@') {
        get_user_by_email(&state.db, &username).await
    } else {
        get_user_by_username(&state.db, &username).await
    };

    let user = match user_result {
        Ok(Some(u)) => u,
        Ok(None) => return (StatusCode::OK, "-2".to_string()),
        Err(e) => {
            eprintln!("login: db error fetching user: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "".to_string());
        }
    };

    let password_ok = bcrypt::verify(&password, &user.password).unwrap_or(false);
    if !password_ok {
        return (StatusCode::OK, "-1".to_string());
    }

    let ip = extract_ip(&headers);
    let (country, city) = get_city(&state.geo, &ip);

    let device_static = form.get("device").cloned().unwrap_or_default();
    let device_dynamic = form.get("deviceDynamic").cloned().unwrap_or_default();
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    if let Err(e) = insert_device(
        &state.db, user.user_id, &ip, &country, &city,
        user_agent, &device_static, &device_dynamic,
    ).await {
        eprintln!("login: insert_device failed: {e}");
        return (StatusCode::INTERNAL_SERVER_ERROR, "".to_string());
    }

    let user_json = serde_json::to_string(&LoginResponse {
        id: user.user_id,
        username: if !user.nickname.as_deref().unwrap_or("").is_empty() {
            user.nickname.unwrap_or_default()
        } else {
            user.username
        },
        is_active: user.activated,
        role: user.priority,
        token: user.token,
        resume: user.resume,
        socials: user.socials,
        city_data: [country, city],
    }).unwrap_or_else(|_| "{}".to_string());

    let body = format!("[{user},[{{}},{{}}],{{}},[]]", user = user_json);

    (StatusCode::OK, body)
}
