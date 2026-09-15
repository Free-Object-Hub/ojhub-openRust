/*
 * ALTCHA реализация
 * Author: Claude
 *
 * На момент создания файла сейчас 14 сентября, 23 часа вечера,
 * я если честно устал от кода и от попыток выучить раст.
 * Возможно мне стоит взять перерыв на пару дней
 *
 * Но с другой стороны, а если после выборов в госдуру у нас отрубят интернет?
 * Хотя тогда нахер мне вообще нужен будет Obect Hub если я к своему же сайту не смогу
 * подсоединиться...
 *
 * Ай ладно нейросети будут жить до ноябрьских выборов в конгресс уж точно,
 * а там после рыночек точно начнёт шататься
 *
 *          ^^^
 * MIOBOMB: Именно по этой причине я предпочитаю обильно вайбкодить и openRust
 */

use altcha_lib_rs::{create_challenge, verify_json_solution, ChallengeOptions};
use axum::{Json, extract::State, http::StatusCode};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::{engine::general_purpose, Engine as _};
use crate::AppState;
use hmac::digest::KeyInit;

// GET /api/altcha/challenge
pub async fn challenge(State(state): State<AppState>) -> Json<serde_json::Value> {
    let ch = create_challenge(ChallengeOptions {
        hmac_key: &state.altcha_secret,
        max_number: Some(100_000),
        ..Default::default()
    }).unwrap();
    Json(serde_json::to_value(ch).unwrap())
}

#[derive(serde::Deserialize)]
pub struct VerifyReq { payload: String }

#[derive(serde::Serialize)]
pub struct VerifyResp { token: String }

// POST /api/altcha/verify
pub async fn verify(State(state): State<AppState>, Json(req): Json<VerifyReq>) -> Result<Json<VerifyResp>, StatusCode> {
    let ok = verify_json_solution(&req.payload, &state.altcha_secret, true).is_ok();
    if !ok { return Err(StatusCode::FORBIDDEN); }

    // выдаём свой токен, отдельный от altcha-протокола — просто HMAC(exp) от того же секрета
    let token = issue_captcha_token(&state.altcha_secret);
    Ok(Json(VerifyResp { token }))
}

// дальше идёт код который уже дёргается в бизнес логике, например при регистрации

fn issue_captcha_token(key: &str) -> String {
    let exp = chrono::Utc::now().timestamp() + 300; // 5 минут на использование
    let msg = exp.to_string();
    let sig = hmac_sign(key, &msg);
    general_purpose::URL_SAFE_NO_PAD.encode(format!("{msg}.{sig}"))
}

// Функция-проверка — вот она, "пропускает работу дальше"
pub fn check_captcha_token(key: &str, token: &str) -> bool {
    let Ok(decoded) = general_purpose::URL_SAFE_NO_PAD.decode(token) else { return false };
    let Ok(s) = String::from_utf8(decoded) else { return false };
    let Some((exp_str, sig)) = s.split_once('.') else { return false };
    if hmac_sign(key, exp_str) != sig { return false }
    exp_str.parse::<i64>().map(|exp| exp > chrono::Utc::now().timestamp()).unwrap_or(false)
}

fn hmac_sign(key: &str, msg: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(key.as_bytes()).unwrap();
    mac.update(msg.as_bytes());
    general_purpose::URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
}
