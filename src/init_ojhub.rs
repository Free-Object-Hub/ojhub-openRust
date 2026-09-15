/* MIOBOMB: Автор этого файла сразу и Claude, и Deepseek
 * Я знаю что контраст абсурдный, но я вспомлин что no всем галлюцинациям deepseek v3 я вообще
 * выучил fastify.js и написал пусть и неработающий но порт ojhub-node
 */

use tokio::try_join;
use indexmap::IndexMap;
use crate::AppState;
use crate::ramdb;
use crate::ojhub::gdpses::{
    get_login_gdpses,
    get_user_gdps_content,
    channel_ids_to_string,
    GdpsShort,
};
use crate::ojhub::news::db::get_global_news;
use crate::wiki::db::get_user_wiki_content;
use crate::user::db::fetch_user_by_token;
use crate::wiki::renders::WikiProfile;
use crate::devices::easy_check_device;

/*
Порт InitOjhub/publicInit/privateInit из openGo (users.go).
Намеренно остаётся "тупым примитивом" — без axum-экстракторов внутри.
Авторизация (кто зовёт, есть ли токен) — забота хендлера, не этой функции.
Здесь только: собрать public+private части параллельно, склеить, закэшировать.
*/

/// Порт extractUserID. Ищет первое `"ID":` в кэшированном JSON и парсит число.
/// Кэш userTc:{token} начинается с private_profile юзера: `{"ID":123,...}`.
fn extract_user_id(cached: &str) -> Option<i32> {
    const PREFIX: &str = r#""ID":"#;
    let start = cached.find(PREFIX)? + PREFIX.len();
    let rest = &cached[start..];
    let end = rest.find([',', '}'])?;
    rest[..end].parse::<i32>().ok()
}

/// Порт injectToken. Вставляет `"token":"..."` первым полем объекта.
/// Трюк: `cached[1..]` отрезает открывающую `{`, а format! подставляет свою.
fn inject_token(cached: &str, token: &str) -> String {
    if token.is_empty() || cached.is_empty() {
        return cached.to_string();
    }
    format!(r#"{{"token":"{token}","#) + &cached[1..]
}

pub async fn init_ojhub(
    state: &AppState,
    ip: &str,
    token: &str,
    device: &str,
    show_token: bool,
    ignore_device: bool,
    bypass_cache: bool,
) -> Result<String, sqlx::Error> {
    let (public, private) = try_join!(
        public_init(state),
        private_init(state, ip, token, device, show_token, ignore_device, bypass_cache),
    )?;

    Ok(format!("[{},{}]", private, public))
}

async fn public_init(state: &AppState) -> Result<String, sqlx::Error> {
    const CACHE_KEY: &str = "loginTcache";

    let cached = ramdb::ram_get(&state.ram, CACHE_KEY).await;
    if !cached.is_empty() {
        return Ok(cached);
    }

    let (gdpses, news) = try_join!(
        get_login_gdpses(&state.db),
        get_global_news(&state.db, 0),
    )?;

    let gdps_map: IndexMap<String, GdpsShort> = gdpses
        .into_iter()
        .map(|p| {
            let s = p.to_short(false, false);
            (format!("{}{}", channel_ids_to_string(s.channel), s.id), s)
        })
        .collect();
    let gdps_json = serde_json::to_string(&gdps_map).unwrap();
    let news_arr: Vec<_> = news.iter().map(|n| n.news_render_legacy()).collect();
    let news_json = serde_json::to_string(&news_arr).unwrap();

    let data = format!("{gdps_json},{news_json}");

    ramdb::ram_set(&state.ram, CACHE_KEY, &data, 600).await;

    Ok(data)
}

async fn private_init(
    state: &AppState,
    ip: &str,
    token: &str,
    device: &str,
    show_token: bool,
    ignore_device: bool,
    bypass_cache: bool,
) -> Result<String, sqlx::Error> {
    if !bypass_cache {
        let cached = ramdb::ram_get(&state.ram, &format!("userTc:{token}")).await;
        if !cached.is_empty() {
            if let Some(uid) = extract_user_id(&cached) {
                let ok = ignore_device || easy_check_device(&state.db, uid, device).await?;
                if ok {
                    return Ok(if show_token { inject_token(&cached, token) } else { cached });
                }
            }
        }
    }

    let city = crate::porting::get_city(&state.geo, ip);

    if token.is_empty() {
        return Ok(great_guest_state(&city, false));
    }

    let Some(user) = fetch_user_by_token(&state.db, token).await? else {
        return Ok(great_guest_state(&city, true));
    };

    if !ignore_device {
        let device_ok = easy_check_device(&state.db, user.id, device).await?;
        if !device_ok {
            return Ok(great_guest_state(&city, true));
        }
    }

    let (gdpses, wikis) = try_join!(
        get_user_gdps_content(&state.db, user.id),
        get_user_wiki_content(&state.db, user.id),
    )?;

    let user_json = serde_json::to_string(&user.private_profile(&city, false)).unwrap();
    let gdps_map: IndexMap<String, GdpsShort> = gdpses
        .into_iter()
        .map(|p| {
            let s = p.to_short(true, true);
            (format!("{}{}", channel_ids_to_string(s.channel), s.id), s)
        })
        .collect();
    let gdps_json = serde_json::to_string(&gdps_map).unwrap();
    let wiki_map: IndexMap<String, WikiProfile> = wikis
        .iter()
        .map(|w| (format!("w{}", w.id), w.to_full()))
        .collect();
    let wiki_json = serde_json::to_string(&wiki_map).unwrap();

    let resp_data = format!("{user_json},[{gdps_json},{wiki_json}]");

    ramdb::ram_set(&state.ram, &format!("userTc:{token}"), &resp_data, 600).await;

    let final_data = if show_token {
        inject_token(&resp_data, token)
    } else {
        resp_data
    };

    Ok(final_data)
}

fn great_guest_state(city: &(String, String), needs_token: bool) -> String {
    let token = if needs_token { "false" } else { "" };
    format!(
        r#"{{"ID":0,"username":"Object Hub","isActive":0,"role":0,"token":"{}","resume":"","socials":"","cityData":["{}","{}"]}},[{{}},{{}}]"#,
        token, city.0, city.1
    )
}
