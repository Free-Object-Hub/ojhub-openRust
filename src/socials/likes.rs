use crate::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json
};
use crate::auth::middleware::RequireDeviceForInactiveAccs;
use std::collections::HashMap;
use sqlx::{FromRow, MySqlPool};

#[derive(FromRow)]
pub struct LikeRow {
    #[sqlx(rename = "whereIz")]
    pub where_iz: i32,
    #[sqlx(rename = "type")]
    pub like_type: i32,
    pub channel: i32,
}

fn like_export_channels() -> HashMap<i32, &'static str> {
    HashMap::from([
        (0, "p"), (1, "c"), (2, "n"), (3, "c"), (4, "c"),
        (5, "c"), (6, "n"), (7, "g"), (8, "w"), (9, "f"),
        (10, "c"), (11, "v"), (12, "c"),
    ])
}

pub async fn fetch_likes(pool: &MySqlPool, user_id: i32) -> Result<Vec<LikeRow>, sqlx::Error> {
    sqlx::query_as::<_, LikeRow>(
        "SELECT whereIz, type, channel FROM likes WHERE userId = ?"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn fetch_subs(pool: &MySqlPool, user_id: i32) -> Result<Vec<i32>, sqlx::Error> {
    sqlx::query_scalar("SELECT gdpsId FROM gdpsSubs WHERE userId = ?")
        .bind(user_id)
        .fetch_all(pool)
        .await
}

pub fn build_likes_response(likes: Vec<LikeRow>, subs: Vec<i32>) -> HashMap<String, Vec<i32>> {
    let channels = like_export_channels();
    let mut result: HashMap<String, Vec<i32>> = [
        "p", "c", "n", "g", "w", "f", "v", "subs",
    ]
    .into_iter()
    .map(|k| (k.to_string(), Vec::new()))
    .collect();
    for l in likes {
        if let Some(ch) = channels.get(&l.channel) {
            result.entry(ch.to_string()).or_default().push(l.where_iz * l.like_type);
        }
    }
    for v in result.values_mut() {
        v.sort_by_key(|x| std::cmp::Reverse(x.abs()));
    }
    result.insert("subs".to_string(), subs);
    result
}

fn empty_result() -> HashMap<String, Vec<i32>> {
    ["p", "c", "n", "g", "w", "f", "v", "subs"]
        .into_iter()
        .map(|k| (k.to_string(), Vec::new()))
        .collect()
}

pub async fn likes_handler(
    RequireDeviceForInactiveAccs(user): RequireDeviceForInactiveAccs,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    if user.activated == 0 {
        return Ok(Json(empty_result()));
    }

    let likes = fetch_likes(&state.db, user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let subs = fetch_subs(&state.db, user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(build_likes_response(likes, subs)))
}
