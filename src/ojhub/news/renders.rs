use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::Serialize;

use super::db::News;
use crate::ojhub::gdpses::channel_ids_to_string;

#[derive(Serialize)]
pub struct NewsResp {
    #[serde(rename = "ID")]
    pub id: i32,
    pub title: String,
    pub text: String,
    #[serde(rename = "author")]
    pub user_id: i32,
    pub username: String,
    #[serde(rename = "gdpsId")]
    pub gdps_id: String,
    #[serde(rename = "gdpsTitle")]
    pub gdps_title: String,
    #[serde(rename = "gdpsImg")]
    pub gdps_img: String,
    pub date: i32,
    pub likes: [i32; 3],
    #[serde(rename = "hasFile")]
    pub has_file: String,
}

/// Base64-декод с фоллбэком на исходную строку, как в Go.
/// Невалидный UTF-8 после декода заменяется replacement-символами
/// (совпадает с поведением Go JSON encoder).
fn decode_text(raw: &str) -> String {
    match BASE64.decode(raw) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        Err(_) => raw.to_string(),
    }
}

/// userName = UserName если непустой, иначе NickName.
/// У нас оба Option из-за LEFT JOIN, обрабатываем как Go-пустые строки.
fn resolve_username(user_name: Option<&str>, nick_name: Option<&str>) -> String {
    match user_name {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => nick_name.unwrap_or("").to_string(),
    }
}

impl News {
    /// Порт News.NewsRender() — структура для новых клиентов.
    pub fn news_render(&self) -> NewsResp {
        let gdps_id = format!(
            "{}{}",
            channel_ids_to_string(self.gdps_channel.unwrap_or(0)),
            self.gdps_id,
        );
        let username = resolve_username(self.user_name.as_deref(), self.nick_name.as_deref());

        NewsResp {
            id: self.id,
            title: self.title.clone(),
            text: decode_text(&self.text),
            user_id: self.user_id,
            username,
            gdps_id,
            gdps_title: self.gdps_title.clone().unwrap_or_default(),
            gdps_img: self.gdps_img.clone().unwrap_or_default(),
            date: self.date,
            likes: [self.likes, self.disls, self.comms_count],
            has_file: self.has_file.clone(),
        }
    }

    /// Порт News.NewsRenderLegacy() — плоский массив из 12 элементов.
    /// Порядок и содержимое зафиксированы клиентами с GDPS Helper 1.X.
    pub fn news_render_legacy(
        &self,
    ) -> (
        i32,         // ID
        String,      // Title
        String,      // Text (после base64)
        i32,         // UserId
        String,      // UserName (или NickName)
        String,      // GdpsId (c123/s234/p345/t456)
        String,      // GdpsTitle
        i32,         // Date
        [i32; 3],    // [Likes, Disls, CommsCount]
        i32,         // isLiked — мёртвое поле, всегда 0
        String,      // HasFile (формат файла)
        String,      // GdpsImg
    ) {
        let gdps_id = format!(
            "{}{}",
            channel_ids_to_string(self.gdps_channel.unwrap_or(0)),
            self.gdps_id,
        );
        let username = resolve_username(self.user_name.as_deref(), self.nick_name.as_deref());

        (
            self.id,
            self.title.clone(),
            decode_text(&self.text),
            self.user_id,
            username,
            gdps_id,
            self.gdps_title.clone().unwrap_or_default(),
            self.date,
            [self.likes, self.disls, self.comms_count],
            0,
            self.has_file.clone(),
            self.gdps_img.clone().unwrap_or_default(),
        )
    }
}
