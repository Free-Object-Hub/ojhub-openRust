use deadpool_redis::{Config, Pool, Runtime, redis::AsyncCommands};
use std::env;
use std::time::Duration;

/// Порт InitRedis. Возвращает None, если редис не поднялся — приложение живёт без кэша.
pub async fn init_redis() -> Option<Pool> {
    let host = env::var("RAMDB_HOST").unwrap_or_else(|_| "localhost".into());
    let port = env::var("RAMDB_PORT").unwrap_or_else(|_| "6379".into());
    let password = env::var("RAMDB_PASSWD").unwrap_or_default();
    let db_num: i32 = env::var("RAMDB_NUM")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    eprintln!("> Connecting to Redis: host={host}, port={port}, db={db_num}");

    let url = if password.is_empty() {
        format!("redis://{host}:{port}/{db_num}")
    } else {
        format!("redis://:{password}@{host}:{port}/{db_num}")
    };

    let pool = match Config::from_url(url).create_pool(Some(Runtime::Tokio1)) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to create Redis pool: {e} (continuing without cache)");
            return None;
        }
    };

    match tokio::time::timeout(Duration::from_secs(5), pool.get()).await {
        Ok(Ok(mut conn)) => {
            if let Err(e) = redis::cmd("PING").query_async::<String>(&mut conn).await {
                eprintln!("Redis PING failed: {e} (continuing without cache)");
                return None;
            }
            eprintln!("> Redis done");
            Some(pool)
        }
        Ok(Err(e)) => {
            eprintln!("Failed to connect to Redis: {e} (continuing without cache)");
            None
        }
        Err(_) => {
            eprintln!("Redis connect timeout (continuing without cache)");
            None
        }
    }
}

/// Порт RamGet. Пустая строка = "нет ключа" ИЛИ "ошибка".
pub async fn ram_get(pool: &Option<Pool>, key: &str) -> String {
    let Some(pool) = pool else { return String::new() };
    let mut conn = match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Redis pool get failed: {e}");
            return String::new();
        }
    };
    match conn.get::<_, Option<String>>(key).await {
        Ok(Some(v)) => v,
        Ok(None) => String::new(),
        Err(e) => {
            eprintln!("Redis GET {key} failed: {e}");
            String::new()
        }
    }
}

/// Порт RamSet. TTL в секундах. Ошибки логируются, но не всплывают.
pub async fn ram_set(pool: &Option<Pool>, key: &str, val: &str, ttl_secs: u64) {
    let Some(pool) = pool else {
        eprintln!("redis is died");
        return;
    };
    let mut conn = match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Redis pool get failed: {e}");
            return;
        }
    };
    if let Err(e) = conn.set_ex::<_, _, ()>(key, val, ttl_secs).await {
        eprintln!("Redis SET {key} failed: {e}");
    }
}

/// Порт RamDel.
pub async fn ram_del(pool: &Option<Pool>, key: &str) {
    let Some(pool) = pool else { return };
    let mut conn = match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Redis pool get failed: {e}");
            return;
        }
    };
    if let Err(e) = conn.del::<_, ()>(key).await {
        eprintln!("Redis DEL {key} failed: {e}");
    }
}
