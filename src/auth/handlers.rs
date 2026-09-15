use axum::{extract::{Form, State}, http::{HeaderMap, StatusCode}, response::IntoResponse};
use std::collections::HashMap;
use crate::auth::exploit_patch;
use crate::porting::get_city;
use crate::user::db::{fetch_user_by_email, fetch_user_by_username};
use altcha_lib_rs::verify_json_solution;
use base64::{engine::general_purpose, Engine as _};
use crate::devices::insert_device;
use crate::porting::extract_ip;
use crate::init_ojhub::init_ojhub;
use crate::user::db::{user_has_used, generate_user_token, generate_user_verify_code, new_user_token, validate_email};

pub async fn register_handler(
    State(state): State<crate::AppState>,
    headers: HeaderMap,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let altcha_payload_raw = form.get("altcha").cloned().unwrap_or_default();
    let altcha_payload = general_purpose::STANDARD
        .decode(&altcha_payload_raw)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_default();

    if altcha_payload.is_empty()
        || verify_json_solution(&altcha_payload, &state.altcha_secret, true).is_err()
    {
        return (StatusCode::OK, "-2".to_string()); // код капчи у регистрации иной, чем у логина (-2, не -3)
    }
    let username = exploit_patch(&form.get("username").cloned().unwrap_or_default());
    let password = form.get("password").cloned().unwrap_or_default();
    let email = exploit_patch(&form.get("email").cloned().unwrap_or_default());
    if username.is_empty() || password.is_empty() {
        return (StatusCode::OK, "-4".to_string());
    }
    match user_has_used(&state.db, &email, &username).await {
        Ok(true) => return (StatusCode::OK, "-4".to_string()),
        Ok(false) => {}
        Err(e) => {
            eprintln!("register: user_has_used error: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "".to_string());
        }
    }
    if !validate_email(&email) {
        return (StatusCode::OK, "-4".to_string());
    }
    let activated = generate_user_verify_code();
    let token = generate_user_token(&username);
    let user = match new_user_token(&state.db, &username, &password, &email, &activated, &token, 0).await {
        Ok(u) => u,
        Err(e) => {
            eprintln!("register: new_user_token error: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "".to_string());
        }
    };
    // TODO: отправка письма (SendUserVerifyMail в Go тоже была под TODO — почта ещё не портирована)
    let ip = extract_ip(&headers);
    let (country, city) = get_city(&state.geo, &ip);
    let device_static = form.get("device").cloned().unwrap_or_default();
    let device_dynamic = form.get("deviceDynamic").cloned().unwrap_or_default();
    let user_agent = headers.get("user-agent").and_then(|v| v.to_str().ok()).unwrap_or("unknown");
    if let Err(e) = insert_device(&state.db, user.id, &ip, &country, &city, user_agent, &device_static, &device_dynamic).await {
        eprintln!("register: insert_device failed: {e}");
        return (StatusCode::INTERNAL_SERVER_ERROR, "".to_string());
    }
    match init_ojhub(&state, &ip, &user.token, &device_static, true, false, false).await {
        Ok(body) => (StatusCode::OK, body),
        Err(e) => {
            eprintln!("register: init_ojhub failed: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "".to_string())
        }
    }
}

pub async fn login_handler(
    State(state): State<crate::AppState>,
    headers: HeaderMap,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let altcha_payload_raw = form.get("altcha").cloned().unwrap_or_default();
    let altcha_payload = general_purpose::STANDARD
        .decode(&altcha_payload_raw)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_default();
    if altcha_payload.is_empty()
        || verify_json_solution(&altcha_payload, &state.altcha_secret, true).is_err()
    {
        return (StatusCode::OK, "-3".to_string());
    }
    let raw_username = form.get("username").cloned().unwrap_or_default();
    let username = exploit_patch(&raw_username);
    let password = form.get("password").cloned().unwrap_or_default();
    let user_result = if username.contains('@') {
        fetch_user_by_email(&state.db, &username).await
    } else {
        fetch_user_by_username(&state.db, &username).await
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
        &state.db, user.id, &ip, &country, &city,
        user_agent, &device_static, &device_dynamic,
    ).await {
        eprintln!("login: insert_device failed: {e}");
        return (StatusCode::INTERNAL_SERVER_ERROR, "".to_string());
    }
    // как в Go: bypass_cache=false, ignore_device=true (мы только что сами создали устройство),
    // show_token=true (клиенту нужен токен именно в ответе на логин)
    match init_ojhub(&state, &ip, &user.token, &device_static, true, true, false).await {
        Ok(body) => (StatusCode::OK, body),
        Err(e) => {
            eprintln!("login: init_ojhub failed: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "".to_string())
        }
    }
}
