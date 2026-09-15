use serde::Serialize;
use sqlx::{FromRow, MySqlPool};

#[derive(Debug, Serialize, FromRow)]
pub struct Wiki {
    #[sqlx(rename = "ID")]
    pub id: i32,
    pub checked: i32,
    #[sqlx(rename = "userId")]
    pub user_id: i32,
    pub title: String,
    pub text: String,
    pub img: String,
    pub language: String,
    pub date: i32,
    pub likes: i32,
    pub disls: i32,
    #[sqlx(rename = "hasLgbt")]
    pub has_lgbt: i32,
    #[sqlx(rename = "connectedGdps")]
    pub connected_gdps: i32,
    #[sqlx(rename = "forumId")]
    pub forum_id: i32,
    #[sqlx(rename = "mainWiki")]
    pub main_wiki: i32,
    pub files: String,
    #[sqlx(rename = "filesSize")]
    pub files_size: i32,
    pub colors: String,
}

pub async fn get_user_wiki_content(db: &MySqlPool, user_id: i32) -> Result<Vec<Wiki>, sqlx::Error> {
    sqlx::query_as::<_, Wiki>(
        r#"
        SELECT DISTINCT w.*
        FROM wikis w
        LEFT JOIN wikisoowners so
            ON w.ID = so.wikiId
            AND so.userId = ?
        WHERE w.userId = ?
            OR so.userId IS NOT NULL
        "#,
    )
    .bind(user_id)
    .bind(user_id)
    .fetch_all(db)
    .await
}
