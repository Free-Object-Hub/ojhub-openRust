/*
 * Object Hub openRust
 * 
 * Contributors:
 * MIOBOMB - архитектура + начало работы + весёлая работа
 * Claude - вся страшная архитектурная возня
 * DenisC - Главный кто тут умеет web_gl на rust
 *
 * Author: MIOBOMB (2026)
 * openRust - не совсем очередная миграция бекенда object hub на другой язык,
 * он скорее является горячей заменой для openGo индекса и загрузчика.
 * Главная причина почему я принялся делать openRust - вебпуши,
 * в webpush-go по слухам всё довольно скудно и плохо работает,
 * а в Rust (если конечно верить Claude) всё с этим в разы лучше.
 * Не знаю, ошибка ли openRust или нет, но я уверен что этот проект ещё большая инвестиция
 * в свои навыки!
 * По крайней мере я наконец начал писать на действительно низкоуровневых языках
 *
 * Смысл openRust останется аналогичный openGo - быть на 100% совместимым
 * с поведением legacy-php, пока никаких амбициозных задач у меня нет вам хватит
 * понимать такую базу
 */

mod loader;
mod db;
mod push;
mod index;

use axum::{Router, routing::get, routing::post};
use loader::cli_loader_handler;
use index::index_handler;
use push::send_push_handler;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;
use sqlx::mysql::MySqlConnectOptions;
use axum::http::StatusCode;
use axum::response::IntoResponse;

static API_ADDR: &str = "/server/133/";
static PHP: &str = ".php";

// то что openGo уже реализовал, но openRust ещё нет -> 398, шлём в openGo
static DROP_TO_OPENGO: &[&str] = &[
    "user/login",
    "user/register",
    "send/newsPost",
    "send/newsModify",
    "delete/newsPost",
    "send/comment",
    "send/commentModify",
    "delete/comment",
    "send/like",
    "send/dislike",
    "reportGdps",
    "user/devices",
    "user/removeDevice",
    "user/getAccInfo",
    "send/deviceAdd",
	"content/getAlarms",
	"content/getAlarm",
	"send/deleteAlarm",
	"send/writeAlarm",
	"gdps/sub",
	"gdps/unsub",
	"gdps/subs",
    "sub",
    "content/fetchComms",
    "content/newsAll",
    "content/news",
    "content/newsC",
    "search/new",
    "wiki/getWikis",
    "wiki/getWiki",
    "wiki/getGuide",
    "vacans/getAll",
    "vacans/apply",
    "vacans/removeApl",
    "content/vacsC",
    "content/camp",
    "loginT",
    "likesT",
];

pub async fn drop_to_go() -> impl IntoResponse {
    (StatusCode::from_u16(398).unwrap(), "")
}

// то что ни openGo ни openRust не реализовали -> 399, сразу в php
static DROP_TO_PHP: &[&str] = &[
    "content/getUser",
    "content/getAddedCamps",
    "content/getAddedShows",
    "content/getAddedPeres",
    "content/getUserGuides",
    "send/campAdd",
    "send/showAdd",
    "send/pereAdd",
    "send/teleAdd",
    "send/campEdit",
    "send/showEdit",
    "send/pereEdit",
    "send/teleEdit",
    "vacans/get",
    "vacans/edit",
    "send/vacsAdd",
    "vacans/removeVac",
    "vacans/applies",
    "content/getJoinLog",
    "search/connectWiki",
    "send/newWiki",
    "send/editWiki",
    "wiki/colors",
    "content/getGuidesAdmin",
    "send/newGuide",
    "send/editGuide",
    "wiki/setWikiTag",
    "wiki/templatesGet",
    "wiki/templateGet",
    "wiki/templateSave",
    "wiki/filesGet",
    "wiki/filesSend",
    "wiki/setMainWiki",
    "content/getOwners",
    "send/perm",
    "send/permAdd",
    "!newTakeAll",
];

pub async fn drop_to_php() -> impl IntoResponse {
    (StatusCode::from_u16(399).unwrap(), "")
}

fn register_fallback_routes(mut app: Router<MySqlPool>) -> Router<MySqlPool> {
    for suffix in DROP_TO_OPENGO {
        let path = format!("{API_ADDR}{suffix}{PHP}");
        app = app.route(&path, axum::routing::any(drop_to_go));
    }
    for suffix in DROP_TO_PHP {
        let path = format!("{API_ADDR}{suffix}{PHP}");
        app = app.route(&path, axum::routing::any(drop_to_php));
    }
    app
}

#[tokio::main]
async fn main() {
    let opts = MySqlConnectOptions::new()
        .host(&std::env::var("SQL_HOST").expect("SQL_HOST"))
        .port(std::env::var("SQL_PORT").expect("SQL_PORT").parse().expect("valid port"))
        .username(&std::env::var("SQL_USER").expect("SQL_USER"))
        .password(&std::env::var("SQL_PASSWD").unwrap_or_default())
        .database(&std::env::var("SQL_DB").expect("SQL_DB"));

    let pool = MySqlPoolOptions::new()
        .max_connections(6)
        .connect_with(opts)
        .await
        .expect("Failed to connect to MySQL");

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/loader", get(cli_loader_handler))
        .route("/cli/send-push", post(send_push_handler));

    let app = register_fallback_routes(app);

    let app = app.with_state(pool);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8087").await.unwrap();
    println!("listening on 8087");
    axum::serve(listener, app).await.unwrap();
}
