use sqlx::MySqlPool;
use super::db::Gdps;

/// Заглушка вместо полного NewGdpsFinder: тот же фиксированный результат
/// (последние 9 проверенных проектов), без параметров.
pub async fn get_login_gdpses(db: &MySqlPool) -> Result<Vec<Gdps>, sqlx::Error> {
    sqlx::query_as::<_, Gdps>(
        r#"
        SELECT *
        FROM gdpses
        WHERE checked = 1
        ORDER BY ID DESC
        LIMIT 9 OFFSET 0
        "#,
    )
    .fetch_all(db)
    .await
}
