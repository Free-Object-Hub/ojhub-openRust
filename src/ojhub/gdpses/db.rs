use crate::utils::{is_valid_lang, exploit_patch};
use sqlx::{MySqlPool, FromRow};
use std::collections::BTreeMap;
use serde::Serialize;

#[derive(FromRow, Serialize, Clone)]
pub struct Gdps {
    #[sqlx(rename = "ID")]
    pub id: i32,
    pub title: String,
    pub description: String,
    pub tags: String,
    pub os: String,
    pub mask: i32,
    pub likes: i32,
    pub disls: i32,
    #[sqlx(rename = "commsCount")]
    pub comms_count: i32,
    pub author: i32,
    pub username: String,
    pub img: String,
    pub ban: String,
    pub channel: i32,
    #[sqlx(rename = "connectedWiki")]
    pub connected_wiki: i32,
    //
    pub link: String,
    pub short: String,
    pub database: String,
    pub checked: i32,
    pub status: i32,
    #[sqlx(rename = "hasLgbt")]
    pub has_lgbt: i32,
    pub points: i32,
    pub freejoin: i32,
    pub language: String,
    #[sqlx(rename = "editCount")]
    pub edit_count: i32,
}

pub struct UserGdpsContent {
    pub owned: Vec<Gdps>,
    pub soowned: Vec<Gdps>,
}

#[derive(FromRow)]
struct GdpsWithRole {
    #[sqlx(flatten)]
    gdps: Gdps,
    role: Option<String>,
}

pub async fn get_all_user_gdps_content(
    pool: &MySqlPool,
    user_id: i32,
) -> Result<UserGdpsContent, sqlx::Error> {
    let rows = sqlx::query_as::<_, GdpsWithRole>(
        r#"
        SELECT 
            g.*,
            CASE
                WHEN g.author = ? THEN 'owner'
                WHEN so.userId IS NOT NULL THEN 'soowner'
            END AS role
        FROM gdpses g
        LEFT JOIN soowners so
            ON g.ID = so.gdpsId
            AND so.userId = ?
        WHERE g.author = ?
            OR so.userId IS NOT NULL
        "#,
    )
    .bind(user_id)
    .bind(user_id)
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut result = UserGdpsContent { owned: Vec::new(), soowned: Vec::new() };
    for row in rows {
        match row.role.as_deref() {
            Some("owner") => result.owned.push(row.gdps),
            Some("soowner") => result.soowned.push(row.gdps),
            _ => {}
        }
    }
    Ok(result)
}

pub async fn get_user_gdps_content(
    pool: &MySqlPool,
    user_id: i32,
) -> Result<Vec<Gdps>, sqlx::Error> {
    sqlx::query_as::<_, Gdps>(
        r#"
        SELECT DISTINCT g.*
        FROM gdpses g
        LEFT JOIN soowners so
            ON g.ID = so.gdpsId
            AND so.userId = ?
        WHERE g.author = ?
            OR so.userId IS NOT NULL
        "#,
    )
    .bind(user_id)
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn gdps_fetch_by_id(pool: &MySqlPool, id: i32) -> Result<Gdps, sqlx::Error> {
    sqlx::query_as::<_, Gdps>("SELECT * FROM gdpses WHERE ID = ?")
        .bind(id)
        .fetch_one(pool)
        .await
}

/// 0 = нет доступа, 1 = совладелец, 2 = владелец
pub async fn check_gdps_access(
    pool: &MySqlPool,
    user_id: i32,
    gdps_id: i32,
) -> Result<i32, sqlx::Error> {
    let (access,): (i32,) = sqlx::query_as(
        r#"
        SELECT CASE
            WHEN EXISTS(
                SELECT 1 FROM gdpses WHERE ID = ? AND author = ?
            ) THEN 2
            WHEN EXISTS(
                SELECT 1 FROM soowners WHERE gdpsId = ? AND userId = ?
            ) THEN 1
            ELSE 0
        END AS access_level
        "#,
    )
    .bind(gdps_id)
    .bind(user_id)
    .bind(gdps_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(access)
}

pub async fn edit_gdps_pictures(
    pool: &MySqlPool,
    gdps_id: i64,
    img: &str,
    ban: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE gdpses SET img = ?, ban = ? WHERE ID = ?")
        .bind(img)
        .bind(ban)
        .bind(gdps_id)
        .execute(pool)
        .await?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum ParseMultiFieldError {
    #[error("malformed field entry: no tab separator")]
    NoSeparator,
    #[error("unknown language: {0}")]
    UnknownLang(String),
    #[error("duplicate language: {0}")]
    DuplicateLang(String),
    #[error("field too long for lang: {0}")]
    TooLong(String),
}

/// Парсит `raw` (записи вида "lang\ttext") в JSON-словарь.
/// BTreeMap — не HashMap: Go's json.Marshal сортирует ключи map по алфавиту,
/// нужно побайтово совпадать при сериализации.
pub fn parse_multi_field(
    raw: &[String],
    max_len: usize,
) -> Result<BTreeMap<String, String>, ParseMultiFieldError> {
    let mut result = BTreeMap::new();
    for entry in raw {
        let idx = entry.find('\t').ok_or(ParseMultiFieldError::NoSeparator)?;
        let lang = exploit_patch(&entry[..idx]);
        let text = exploit_patch(&entry[idx + 1..]);

        if !is_valid_lang(&lang) {
            return Err(ParseMultiFieldError::UnknownLang(lang));
        }
        if result.contains_key(&lang) {
            return Err(ParseMultiFieldError::DuplicateLang(lang));
        }
        if text.len() > max_len {
            return Err(ParseMultiFieldError::TooLong(lang));
        }
        if text.is_empty() {
            continue;
        }
        result.insert(lang, text);
    }
    Ok(result)
}
