use sqlx::MySqlPool;
use woothee::parser::Parser;

pub async fn check_device(
    pool: &MySqlPool,
    user_id: i32,
    user_agent: &str,
    country: &str,
    city: &str,
    static_fp: &str,
) -> Result<bool, sqlx::Error> {
    let exists: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM devices WHERE userId = ? AND userAgent = ? AND country = ? AND city = ? AND staticFp = ?)"
    )
    .bind(user_id)
    .bind(user_agent)
    .bind(country)
    .bind(city)
    .bind(static_fp)
    .fetch_one(pool)
    .await?;

    Ok(exists.0)
}

pub async fn add_device(
    pool: &MySqlPool,
    user_id: i32,
    user_agent: &str,
    ip: &str,
    country: &str,
    city: &str,
    platform: &str,
    browser: &str,
    static_fp: &str,
    dynamic_fp: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO devices (userId, userAgent, ip, country, city, platform, browser, staticFp, dynamicFp) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(user_id)
    .bind(user_agent)
    .bind(ip)
    .bind(country)
    .bind(city)
    .bind(platform)
    .bind(browser)
    .bind(static_fp)
    .bind(dynamic_fp)
    .execute(pool)
    .await?;

    Ok(())
}

fn parse_ua(user_agent: &str) -> (String, String) {
    let parser = Parser::new();
    match parser.parse(user_agent) {
        Some(result) => {
            let platform = format!("{} {}", result.os, result.os_version);
            let browser = format!("{} {}", result.name, result.version);
            (platform.trim().to_string(), browser.trim().to_string())
        }
        None => ("Unknown".to_string(), "Unknown".to_string()),
    }
}

pub async fn insert_device(
    pool: &MySqlPool,
    user_id: i32,
    ip: &str,
    country: &str,
    city: &str,
    user_agent: &str,
    static_fp: &str,
    dynamic_fp: &str,
) -> Result<(), sqlx::Error> {
    let exists = check_device(pool, user_id, user_agent, country, city, static_fp).await?;
    if exists {
        return Ok(());
    }

    let (platform, browser) = parse_ua(user_agent);

    add_device(
        pool, user_id, user_agent, ip, country, city,
        &platform, &browser,
        static_fp, dynamic_fp,
    ).await
}
