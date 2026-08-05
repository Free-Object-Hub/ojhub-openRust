use axum::{
    extract::{
        State,
        Query,
        RawQuery
    }, http::{
        HeaderMap,
        header
    }, response::{
        Html,
        IntoResponse,
        Response
    }
};
use std::collections::HashMap;
use sqlx::MySqlPool;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use crate::db::{gdps_fetch_by_id, wiki_fetch_by_id, vac_fetch_by_id, news_fetch_by_id};
use crate::loader::get_ver_from_cookie;

fn build_description(short: Option<&str>, fallback: &str) -> String {
    match short {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => truncate_chars(fallback, 120),
    }
}

fn truncate_chars(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}

async fn build_meta_tags(pool: &MySqlPool, params: &HashMap<String, String>, raw_query: &Option<String>) -> String {
    if let Some((route, id)) = parse_future_link_format(raw_query) {
        if route == "VacsC" {
            if let Ok(Some(vac)) = vac_fetch_by_id(pool, id).await {
                let descr = build_description(vac.short.as_deref(), &vac.text);
                let image = vac.g_title.map(|_| "".to_string()).unwrap_or_default(); 
                // ^ картинки у вакансии нет в структуре, но можешь добавить gImg в SELECT если нужна
                return build_tags(&vac.title, &descr, &image);
            }
        }
    }
    if params.contains_key("Wikis") {
        return format!(
            r#"<meta property="og:title" content="Object Hub Wiki">
            <meta property="og:description" content="Object Hub Wiki - не Mediawiki! добро пожаловать на наш вики движок!">
            <meta property="og:image" content="https://objecthub.xyz/imgs/hubbig.png">"#
        );
    }
    if let Some(wiki_id) = params.get("wiki").and_then(|s| s.parse::<i32>().ok()) {
        if let Ok(Some(wiki)) = wiki_fetch_by_id(pool, wiki_id).await {
            let descr = build_description(wiki.short.as_deref(), &wiki.text);
            return build_tags(&wiki.title, &descr, &wiki.img);
        }
    }
    let gdps_id = ["camp", "show", "pere", "tele"]
        .iter()
        .find_map(|key| params.get(*key))
        .and_then(|s| s.parse::<i32>().ok());
    if let Some(id) = gdps_id {
        if let Ok(Some(gdps)) = gdps_fetch_by_id(pool, id).await {
            let descr = build_description(gdps.short.as_deref(), &gdps.description);
            return build_tags(&gdps.title, &descr, &gdps.img);
        }
    }
    if let Some(news_param) = params.get("news/comms") {
        let id_part = news_param.split('|').next().unwrap_or("");
        if let Ok(news_id) = id_part.parse::<i32>() {
            if let Ok(Some(news)) = news_fetch_by_id(pool, news_id).await {
                let decoded = STANDARD.decode(&news.text)
                    .ok()
                    .and_then(|bytes| String::from_utf8(bytes).ok())
                    .unwrap_or_default();
                let descr = truncate_chars(&decoded, 120);
                let image = news.g_img.unwrap_or_default();
                return build_tags(&news.title, &descr, &image);
            }
        }
    }
    build_tags(
        "Object hub",
        "Удобный сервис для поиска и размещения своих обджект шоу и кемпов!",
        "https://objecthub.xyz/imgs/hubbig.png",
    )
}

fn parse_future_link_format(raw: &Option<String>) -> Option<(String, i32)> {
    let raw = raw.as_ref()?;
    for pair in raw.split('&') {
        if let Some((route, id_str)) = pair.split_once('/') {
            if let Ok(id) = id_str.parse::<i32>() {
                return Some((route.to_string(), id));
            }
        }
    }
    None
}

fn build_tags(title: &str, desc: &str, img: &str) -> String {
    format!(
        r#"<meta property="og:title" content="{}">
        <meta property="og:description" content="{}">
        <meta property="og:image" content="{}">"#,
        title, desc, img
    )
}

pub async fn index_handler(
    State(pool): State<MySqlPool>,
    Query(params): Query<HashMap<String, String>>,
    RawQuery(raw_query): RawQuery,
    headers: HeaderMap
) -> Response {
    let ver = get_ver_from_cookie(&headers);
    
    let meta_tags = build_meta_tags(&pool, &params, &raw_query).await;
    
    let html = format!(r#"<!DOCTYPE html>
<html>
    <head>
        <meta name=viewport content="width=device-width,initial-scale=1.0">
        <meta charset=UTF-8>
        {meta}
        <title>Object Hub</title>
        <link rel=icon>
        {ver}
        <style id=wikiStyle></style>
    </head>
    <body style="background-color:var(--color-bg)">
        <div id=1st></div>
        <div id=windowsXP>
            <div id=Professional class=hider></div>
        </div>
        <div id=alerts class=alerts></div>
    </body>
</html>"#, meta = meta_tags, ver = ver.extra);
    
    (
        [(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")],
        Html(html),
    ).into_response()
}
