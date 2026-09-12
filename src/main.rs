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
 *
 * И да, я как не любил бекенды, так их и не люблю, но совместимость есть совсемтимость
 */

mod loader;
mod db;
mod push;
mod index;
mod auth;
mod user;
mod devices;
mod porting;
mod utils;

use axum::{Router, routing::get, routing::post};
use loader::cli_loader_handler;
use index::index_handler;
use push::send_push_handler;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;
use sqlx::mysql::MySqlConnectOptions;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use maxminddb::Reader;
use std::sync::Arc;

static API_ADDR: &str = "/server/133/";
static PHP: &str = ".php";

// то что openGo уже реализовал, но openRust ещё нет -> 398, шлём в openGo
static DROP_TO_OPENGO: &[&str] = &[
    "user/login",
    "user/logout",
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
    "user/setNickname",
    "user/setSocials",
    "user/setResume",
    "send/deviceAdd",
	"content/getAlarms",
	"content/getAlarm",
	"send/deleteAlarm",
	"send/writeAlarm",
    "send/campAdd",
    "send/showAdd",
    "send/pereAdd",
    "send/teleAdd",
    "send/campEdit",
    "send/showEdit",
    "send/pereEdit",
    "send/teleEdit",
    "content/getJoinLog",
    "send/bump",
	"content/getOwners",
	"send/permAdd",
	"send/perm",
	"gdps/sub",
	"gdps/unsub",
	"gdps/subs",
    "sub",
    "content/fetchComms",
    "content/newsAll",
    "content/news",
    "content/newsC",
    "search/new",
    "content/getUser",
    "content/getAddedCamps",
    "content/getAddedShows",
    "content/getAddedPeres",
    "content/getAddedTeles",
    "content/getUserGuides",
    "wiki/getWikis",
    "wiki/getWiki",
    "wiki/getGuides",
    "wiki/getGuide",
    "send/newWiki",
    "send/editWiki",
    "wiki/colors",
    "wiki/setMainWiki",
    "search/conntectWiki",
    "search/conntectContent",
    "content/getGuidesAdmin",
    "send/newGuide",
    "send/editGuide",
    "wiki/setWikiTag",
    "wiki/templateGet",
    "wiki/templatesGet",
    "wiki/templateSave",
    "wiki/templateDelete",
    "forum/create",
    "forum/createPost",
    "send/forumPost",
    "forum/getPost",
    "forum/getPosts",
    "vacans/getAll",
    "vacans/apply",
    "vacans/removeApl",
    "vacans/get",
    "vacans/edit",
    "vacans/removeVac",
    "vacans/applies",
    "send/vacsAdd",
    "send/vacsEdit",
    "content/vacsC",
    "content/camp",
    "wordleRU",
    "wordleEN",
    "loginT",
    "likesT",
    "!newTakeAll",
    "Aaction",
];

pub async fn drop_to_go() -> impl IntoResponse {
    (StatusCode::from_u16(398).unwrap(), "")
}

// то что ни openGo ни openRust не реализовали -> 404, legacy php выключен же
static DROP_TO_PHP: &[&str] = &[
    "wiki/filesGet",
    "wiki/filesSend",
    "search/deleteWikiFiles",
    "vless", // MIOBOMB: я использую это для себя на локалхосте, забыл удалить лол
];

pub async fn drop_to_php() -> impl IntoResponse {
    (StatusCode::from_u16(399).unwrap(), "")
}

fn register_fallback_routes(mut app: Router<AppState>) -> Router<AppState> {
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

#[derive(Clone)]
pub struct AppState {
    pub db: MySqlPool,
    pub geo: std::sync::Arc<maxminddb::Reader<Vec<u8>>>,
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
    let geo_reader = Reader::open_readfile("GeoLite2-City.mmdb")
        .expect("Failed to load GeoLite2 database");
    let state = AppState {
        db: pool,
        geo: Arc::new(geo_reader),
    };

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/loader", get(cli_loader_handler))
        .route("/cli/send-push", post(send_push_handler));

    let app = register_fallback_routes(app);

    let app = app.with_state(state);
    
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8087").await.unwrap();
    println!("listening on 8087");
    axum::serve(listener, app).await.unwrap();
}
