use serde::Serialize;
use sqlx::{FromRow, MySqlPool};

#[derive(Debug, Serialize, FromRow)]
pub struct News {
    #[sqlx(rename = "ID")]
    pub id: i32,
    #[sqlx(rename = "gdpsId")]
    pub gdps_id: i32,
    pub title: String,
    pub text: String,
    #[sqlx(rename = "gTitle")]
    pub gdps_title: Option<String>,
    #[sqlx(rename = "gImg")]
    pub gdps_img: Option<String>,
    #[sqlx(rename = "gChannel")]
    pub gdps_channel: Option<i32>,
    #[sqlx(rename = "userId")]
    pub user_id: i32,
    #[sqlx(rename = "uUsername")]
    pub user_name: Option<String>,
    #[sqlx(rename = "uNickname")]
    pub nick_name: Option<String>,
    pub date: i32,
    pub likes: i32,
    pub disls: i32,
    pub checked: i32,
    #[sqlx(rename = "commsCount")]
    pub comms_count: i32,
    #[sqlx(rename = "hasFile")]
    pub has_file: String,
    #[sqlx(default)]
    pub username: Option<String>,
}

pub async fn get_global_news(db: &MySqlPool, page: i32) -> Result<Vec<News>, sqlx::Error> {
    let offset = if page <= 0 { 0 } else { page * 10 };

    sqlx::query_as::<_, News>(
        r#"
        SELECT n.*, u.username as uUsername, u.nickname as uNickname,
               g.title as gTitle, g.img as gImg, g.channel as gChannel
        FROM news n
        LEFT JOIN users u ON n.userId = u.userId
        LEFT JOIN gdpses g ON n.gdpsId = g.ID
        WHERE n.checked = 1
        ORDER BY n.ID DESC LIMIT 11 OFFSET ?
        "#,
    )
    .bind(offset)
    .fetch_all(db)
    .await
}

pub async fn news_fetch_by_id(db: &MySqlPool, id: i32) -> Result<Option<News>, sqlx::Error> {
    sqlx::query_as::<_, News>(
        r#"
        SELECT n.*, u.username as uUsername, u.nickname as uNickname,
               g.title as gTitle, g.img as gImg, g.channel as gChannel
        FROM news n
        LEFT JOIN users u ON n.userId = u.userId
        LEFT JOIN gdpses g ON n.gdpsId = g.ID
        WHERE n.ID = ?
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn get_local_news(
    db: &MySqlPool,
    gdps_id: i32,
    page: i32,
) -> Result<Vec<News>, sqlx::Error> {
    let offset = if page <= 0 { 0 } else { page * 10 };

    sqlx::query_as::<_, News>(
        r#"
        SELECT n.*, u.username as uUsername, u.nickname as uNickname,
               g.title as gTitle, g.img as gImg, g.channel as gChannel
        FROM news n
        LEFT JOIN users u ON n.userId = u.userId
        LEFT JOIN gdpses g ON n.gdpsId = g.ID
        WHERE n.gdpsId = ?
        ORDER BY n.ID DESC LIMIT 11 OFFSET ?
        "#,
    )
    .bind(gdps_id)
    .bind(offset)
    .fetch_all(db)
    .await
}

pub async fn news_post(
    db: &MySqlPool,
    user_id: i32,
    gdps_id: i32,
    text: &str,
    date: i32,
    title: &str,
    checked: i32,
    has_file: &str,
) -> Result<i32, sqlx::Error> {
    let res = sqlx::query(
        "INSERT INTO news (userId, gdpsId, date, title, text, checked, hasFile) VALUES (?,?,?,?,?,?,?)",
    )
    .bind(user_id)
    .bind(gdps_id)
    .bind(date)
    .bind(title)
    .bind(text)
    .bind(checked)
    .bind(has_file)
    .execute(db)
    .await?;
    Ok(res.last_insert_id() as i32)
}

pub async fn news_edit(
    db: &MySqlPool,
    id: i32,
    text: &str,
    title: &str,
    gdps_id: i32,
) -> Result<u64, sqlx::Error> {
    let res = sqlx::query(
        "UPDATE `news` SET `text` = ?, `title` = ? WHERE `ID` = ? AND gdpsId = ?",
    )
    .bind(text)
    .bind(title)
    .bind(id)
    .bind(gdps_id)
    .execute(db)
    .await?;
    Ok(res.rows_affected())
}

pub async fn news_delete(db: &MySqlPool, id: i32) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;
    sqlx::query("DELETE FROM likes WHERE whereIz = ? AND channel = 6")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM news WHERE ID = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}
