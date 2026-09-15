/*
ojhub@localhost [objecthub]> DESCRIBE users;
+-----------+--------------+------+-----+---------+----------------+
| Field     | Type         | Null | Key | Default | Extra          |
+-----------+--------------+------+-----+---------+----------------+
| userId    | int(11)      | NO   | PRI | NULL    | auto_increment |
| username  | varchar(255) | NO   |     | NULL    |                |
| nickname  | varchar(255) | NO   |     |         |                |
| commTime  | int(11)      | NO   |     | 0       |                |
| password  | varchar(255) | NO   |     | NULL    |                |
| mail      | varchar(255) | NO   |     | NULL    |                |
| activated | int(11)      | NO   |     | 0       |                |
| code      | varchar(6)   | NO   |     | 000000  |                |
| priority  | tinyint(4)   | NO   |     | NULL    |                |
| token     | varchar(128) | NO   |     | NULL    |                |
| resume    | mediumtext   | NO   |     | ''      |                |
| socials   | mediumtext   | NO   |     | ''      |                |
+-----------+--------------+------+-----+---------+----------------+
12 rows in set (0.001 sec)
*/
use sqlx::{MySqlPool, FromRow};
use rand::Rng;
use rand::RngExt;
use sha2::{Sha256, Digest};

#[derive(FromRow, Clone)]
pub struct User {
    #[sqlx(rename = "userId")]
    pub id: i32,
    pub username: String,
    pub nickname: String,
    #[sqlx(rename = "commTime")]
    pub comm_time: i32,
    pub password: String,
    pub mail: String,
    pub activated: i32,
    pub code: String,
    pub priority: i32,
    pub token: String,
    pub resume: String,
    pub socials: String,
}

pub async fn fetch_user_by_id(pool: &MySqlPool, id: i32) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT userId, username, nickname, commTime, password, mail, activated, code, priority, token, resume, socials FROM users WHERE userId = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn fetch_user_by_email(pool: &MySqlPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT userId, username, nickname, commTime, password, mail, activated, code, priority, token, resume, socials FROM users WHERE mail = ?")
        .bind(email)
        .fetch_optional(pool)
        .await
}

pub async fn fetch_user_by_token(pool: &MySqlPool, token: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT userId, username, nickname, commTime, password, mail, activated, code, priority, token, resume, socials FROM users WHERE token = ?")
        .bind(token)
        .fetch_optional(pool)
        .await
}

pub async fn fetch_user_by_username(pool: &MySqlPool, username: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT userId, username, nickname, commTime, password, mail, activated, code, priority, token, resume, socials FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await
}

pub async fn user_has_used(db: &MySqlPool, email: &str, username: &str) -> Result<bool, sqlx::Error> {
    let exists: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM users WHERE mail = ? OR username = ?)"
    )
    .bind(email)
    .bind(username)
    .fetch_one(db)
    .await?;
    Ok(exists.0)
}

fn random_string(length: usize) -> String {
    const CHARSET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut rng = rand::rng();
    (0..length)
        .map(|_| CHARSET[rng.random_range(0..CHARSET.len())] as char)
        .collect()
}

pub fn generate_user_token(username: &str) -> String {
    let entropy = random_string(16);
    let mut hasher = Sha256::new();
    hasher.update(format!("{entropy}{username}").as_bytes());
    hex::encode(hasher.finalize())
}

pub fn generate_user_verify_code() -> String {
    let mut rng = rand::rng();
    format!("{:06}", rng.random_range(0..1_000_000u32))
}

pub fn validate_email(email: &str) -> bool {
    // порт ValidateEmail: без CR/LF, непустой, парсится как валидный адрес
    let email = email.trim();
    if email.is_empty() || email.contains('\r') || email.contains('\n') {
        return false;
    }
    // без net/mail-эквивалента в std — простая, но достаточная проверка формата
    let Some((local, domain)) = email.split_once('@') else { return false };
    !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
}

pub async fn new_user_token(
    db: &MySqlPool,
    username: &str,
    password: &str,
    email: &str,
    activated: &str,
    token: &str,
    priority: i32,
) -> Result<User, sqlx::Error> {
    let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

    let result = sqlx::query(
        "INSERT INTO users (username, password, mail, code, token, priority) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(username)
    .bind(&password_hash)
    .bind(email)
    .bind(activated)
    .bind(token)
    .bind(priority)
    .execute(db)
    .await?;

    Ok(User {
        id: result.last_insert_id() as i32,
        username: username.to_string(),
        nickname: String::new(),
        comm_time: 0,
        password: password_hash,
        mail: email.to_string(),
        activated: 0, // как в Go: сам INSERT не пишет activated-число, activated там = "код" (0/1, ниже нюанс)
        code: activated.to_string(),
        priority,
        token: token.to_string(),
        resume: String::new(),
        socials: String::new(),
    })
}
