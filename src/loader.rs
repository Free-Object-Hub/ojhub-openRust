/*
 * Open Loader 1.20
 *
 * Author: MIOBOMB + Claude
 *
 * Ojhub Loader, штука которая подставляет в index роут нужные html теги для загрузки
 * разных версий сайта. Ниже идёт ченжлог.
 *
 * Open Loader 1.10:
 * В отличии от nodejs загрузчика он умеет подставлять кастомные теги для каждой версии,
 * отсюда версия 1.10 в openGo.
 * Для понимания в nodejs "/" маршруте все теги были захардкожены и в них просто подставлялась
 * кука с версией, это ломало 0.97.33 в Chromium браузерах, здесь такого уже нет.
 *
 * Open Loader 1.20
 * Я удалил из таблицы мёртвые версии!
 * причина до боли проста но неочевидна - зачем мне помнить про эти версии, если
 * в загрузчике даже нет ченжлога к ним?
 * А если серьёзно, в Open Loader 1.30 я сделаю отдельную таблицу с мёртвыми версиями
 * и добавлю к каждой версии ченжлог который открывается в newHelper win окне
 */

use axum::{http::HeaderMap, response::Html};

pub fn default_ver() -> String {
    std::env::var("CLI_VER").unwrap_or_else(|_| "0.97.7".to_string())
}

pub fn default_version() -> &'static ClientVersion {
    find_version(&default_ver()).expect("CLI_VER must exist in VERSIONS")
}

pub fn find_version(ver: &str) -> Option<&'static ClientVersion> {
    VERSIONS.iter().find(|v| v.ver == ver)
}

pub struct ClientVersion {
    pub ver: &'static str,
    pub date: &'static str,
    pub desc: &'static str,
    pub extra: &'static str,
}

pub static VERSIONS: &[ClientVersion] = &[

    ClientVersion {
        ver: "0.97.8", date: "?? ??? 2026", desc: "openRust AND action write init",
        extra: r#"<link href="./cli/0.97.8/main.css?ver=20" rel=stylesheet>
        <link href="./cli/0.97.8/window.css?ver=20" rel=stylesheet>
        <script defer src="./cli/0.97.8/newHelper.js?ver=25"></script>
        <script defer src="./cli/0.97.8/nhConfig.js?ver=25"></script>
        <script defer src="./cli/0.97.8/ojhub.js?ver=25"></script>"#
    },

    ClientVersion {
        ver: "0.97.7", date: "27 Jul 2026", desc: "openGo init",
        extra: r#"<link href="./cli/0.97.7/main.css?ver=20" rel=stylesheet>
        <link href="./cli/0.97.7/window.css?ver=20" rel=stylesheet>
        <script defer src="./cli/0.97.7/newHelper.js?ver=25"></script>
        <script defer src="./cli/0.97.7/nhConfig.js?ver=25"></script>
        <script defer src="./cli/0.97.7/ojhub.js?ver=25"></script>"#
    },

    ClientVersion {
        ver: "0.97.33", date: "31 Jan 2026", desc: "",
        extra: r#"<link rel="preconnect" href="https://fonts.googleapis.com">
        <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
        <link href="https://fonts.googleapis.com/css2?family=Comfortaa:wght@300..700&family=Unbounded:wght@200..900&display=swap" rel="stylesheet">
        <link href="https://fonts.googleapis.com/css2?family=Comfortaa:wght@300..700&family=Huninn&family=Manrope:wght@200..800&family=News+Cycle:wght@400;700&family=Unbounded:wght@200..900&display=swap" rel="stylesheet">
        <link href="./cli/0.97.33/main.css?ver=20" rel=stylesheet>
        <link href="./cli/0.97.33/window.css?ver=20" rel=stylesheet>
        <script defer src="./cli/0.97.33/newHelper.js?ver=21"></script>
        <script defer src="./cli/0.97.33/nhConfig.js?ver=21"></script>"#
    },

    ClientVersion {
        ver: "0.96.3", date: "12 Sep 2025", desc: "wiki isnt working",
        extra: r#"<link href="./cli/0.96.3/main.css?ver=18" rel=stylesheet>
        <link href="./cli/0.96.3/window.css?ver=18" rel=stylesheet>
        <script defer src="./cli/0.96.3/ojhub.js?ver=18&helper"></script>"#
    },

    ClientVersion {
        ver: "GHE1.9", date: "24 Now 2024", desc: "GDPS Helper 1.901, not object hub",
        extra: r#"<link href="./cli/GHE1.9/main.css" rel=stylesheet>
        <style id="stule">
            :root {
                --color-main:rgb(157,97,42);
                --color-light:rgb(255,134,0);
                --color-weekly:rgb(189,99,0);
                --color-weekly-alpha:rgba(189,99,0,.6);
                --color-black:rgb(29,28,22);
                --color-black-alpha:rgba(29,28,22,.6);
                --color-profile:rgb(32,31,24);
                --color-profile-alpha:rgb(32,31,24,.6);
            }
        </style>
        <script defer src="./cli/GHE1.9/newHelper.js"></script>
        <script defer>
            setTimeout(()=>document.body.style="background-color:rgb(12,12,3)",100)
        </script>
        "#
    },

];

pub async fn cli_loader_handler(headers: HeaderMap) -> Html<String> {
    let current = get_ver_from_cookie(&headers);
    
    let mut rows = String::new();
    rows.push_str(
    r#"<tr>
            <td><button onclick="(document.cookie='cli_ver=;path=/;max-age=0');location.pathname=''">stable</button></td>
            <td></td>
            <td></td>
        </tr>"#
    );
    for v in VERSIONS.iter() {
        rows.push_str(&format!(
            r#"<tr>
                <td><button onclick="(document.cookie='cli_ver={ver}; path=/; max-age={max_age}');location.pathname=''">{ver}</button></td>
                <td>{date}</td>
                <td>{desc}</td>
            </tr>"#,
            ver = v.ver,
            max_age = 60 * 60 * 24 * 365,
            date = v.date,
            desc = v.desc
        ));
    }
    
    //FIXME: сделать current значением куки пользователя
    let html = format!(r#"<!DOCTYPE html>
<html>
<body style="display:flex;justify-content:center;align-items:center;min-height:100vh;flex-direction:column">
    <h1>OJHUB LOADER v1.20</h1>
    <p>selected: {current}</p>
    <table border=1>
        <tr>
            <th>ver</th>
            <th>date</th>
            <th>desc</th>
        </tr>
        {rows}
    </table>
</body>
</html>"#, current = current.ver, rows = rows);
    
    Html(html)
}

pub fn get_ver_from_cookie(headers: &HeaderMap) -> &'static ClientVersion {
    let cookie_header = match headers.get("cookie") {
        Some(v) => v.to_str().unwrap_or(""),
        None => return default_version(),
    };
    for part in cookie_header.split(';') {
        let part = part.trim();
        if let Some(val) = part.strip_prefix("cli_ver=") {
            if let Some(cv) = find_version(val) {
                return cv;
            }
        }
    }
    default_version()
}
