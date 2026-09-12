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
