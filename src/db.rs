use sqlx::{MySqlPool, FromRow};

#[derive(FromRow)]
pub struct GdpsMeta {
    #[sqlx(rename = "ID")]
    pub id: i32,
    pub title: String,
    pub description: String,
    pub img: String,
    pub short: Option<String>,
}

pub async fn gdps_fetch_by_id(pool: &MySqlPool, id: i32) -> Result<Option<GdpsMeta>, sqlx::Error> {
    sqlx::query_as::<_, GdpsMeta>("SELECT ID, title, description, img, short FROM gdpses WHERE ID = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

#[derive(FromRow)]
pub struct WikiMeta {
    #[sqlx(rename = "ID")]
    pub id: i32,
    pub title: String,
    pub text: String,
    pub img: String,
    pub short: Option<String>,
}

pub async fn wiki_fetch_by_id(pool: &MySqlPool, id: i32) -> Result<Option<WikiMeta>, sqlx::Error> {
    sqlx::query_as::<_, WikiMeta>("SELECT ID, title, text, img, short FROM wikis WHERE ID = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

#[derive(FromRow)]
pub struct VacancyMeta {
    #[sqlx(rename = "ID")]
    pub id: i32,
    pub title: String,
    pub text: String,
    pub short: Option<String>,
    #[sqlx(rename = "gChannel")]
    pub g_channel: Option<i32>,
    #[sqlx(rename = "gTitle")]
    pub g_title: Option<String>,
}

pub async fn vac_fetch_by_id(pool: &MySqlPool, vac_id: i32) -> Result<Option<VacancyMeta>, sqlx::Error> {
    sqlx::query_as::<_, VacancyMeta>(
        r#"SELECT v.ID, v.title, v.text, v.short, g.channel AS gChannel, g.title AS gTitle
        FROM vacans v
        LEFT JOIN gdpses g ON v.gdpsId = g.ID
        WHERE v.ID = ?"#
    )
    .bind(vac_id)
    .fetch_optional(pool)
    .await
}

#[derive(FromRow)]
pub struct NewsMeta {
    #[sqlx(rename = "ID")]
    pub id: i32,
    pub title: String,
    pub text: String,
    #[sqlx(rename = "gTitle")]
    pub g_title: Option<String>,
    #[sqlx(rename = "gImg")]
    pub g_img: Option<String>,
}

pub async fn news_fetch_by_id(pool: &MySqlPool, news_id: i32) -> Result<Option<NewsMeta>, sqlx::Error> {
    sqlx::query_as::<_, NewsMeta>(
        r#"SELECT n.ID, n.title, n.text, n.short, g.title AS gTitle, g.img AS gImg
        FROM news n
        LEFT JOIN gdpses g ON n.gdpsId = g.ID
        WHERE n.ID = ?"#
    )
    .bind(news_id)
    .fetch_optional(pool)
    .await
}
