use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use super::db::Gdps;
use crate::utils::{truncate_for_preview, PREVIEW_TRUNCATE_LIMIT};

#[derive(Serialize)]
pub struct GdpsShort {
    #[serde(rename = "ID")]
    pub id: i32,
    pub title: String,
    pub text: String,
    pub tags: String,
    pub os: String,
    pub likes: [i32; 3], // 0: лайки, 1: дизлайки, 2: число комментариев
    pub author: i32,
    pub username: String,
    pub img: String,
    pub ban: String,
    pub channel: i32,
    pub wiki: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checked: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points: Option<i32>,
}

impl Gdps {
    pub fn to_short(&self, full_text: bool, render_checked: bool) -> GdpsShort {
        let text = if full_text {
            self.description.clone()
        } else if !self.short.is_empty() {
            self.short.clone()
        } else if self.description.len() > PREVIEW_TRUNCATE_LIMIT {
            truncate_for_preview(&self.description, PREVIEW_TRUNCATE_LIMIT)
        } else {
            self.description.clone()
        };

        let (checked, points) = if render_checked {
            (Some(self.checked), Some(self.points))
        } else {
            (None, None)
        };

        GdpsShort {
            id: self.id,
            title: self.title.clone(),
            text,
            tags: self.tags.clone(),
            os: self.os.clone(),
            likes: [self.likes, self.disls, self.comms_count],
            author: self.author,
            username: self.username.clone(),
            img: self.img.clone(),
            ban: self.ban.clone(),
            channel: self.channel,
            wiki: self.connected_wiki,
            checked,
            points,
        }
    }
}

#[derive(Serialize)]
pub struct GdpsFull {
    #[serde(rename = "ID")]
    pub id: i32,
    pub title: String,
    pub text: String,
    pub tags: String,
    pub os: String,
    pub links: Value,
    pub likes: [i32; 3], // 0: лайки, 1: дизлайки, 2: число комментариев
    pub author: i32,
    pub username: String,
    pub img: String,
    pub ban: String,
    pub freejoin: i32,
    pub language: String,
    pub channel: i32,
    pub wiki: i32,
}

/// Костыль ради совместимости с проектами, у которых ссылка ещё в формате
/// простой строки, а не JSON.
pub fn parse_gdps_links(raw: &str) -> Value {
    let cleaned = raw.replace(r#"\""#, "\"");
    if cleaned.starts_with('{') {
        if let Ok(parsed) = serde_json::from_str::<Value>(&cleaned) {
            return parsed;
        }
    }
    Value::String(cleaned)
}

impl Gdps {
    pub fn to_full(&self) -> GdpsFull {
        GdpsFull {
            id: self.id,
            title: self.title.clone(),
            text: self.description.clone(),
            tags: self.tags.clone(),
            os: self.os.clone(),
            links: parse_gdps_links(&self.link),
            likes: [self.likes, self.disls, self.comms_count],
            author: self.author,
            username: self.username.clone(),
            img: self.img.clone(),
            ban: self.ban.clone(),
            freejoin: self.freejoin,
            language: self.language.clone(),
            channel: self.channel,
            wiki: self.connected_wiki,
        }
    }
}
