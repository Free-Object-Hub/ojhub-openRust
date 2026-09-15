use serde::Serialize;
use super::db::Wiki;

#[derive(Serialize)]
pub struct WikiShort {
    #[serde(rename = "ID")]
    pub id: i32,
    pub title: String,
    pub text: String,
    #[serde(rename = "ban")]
    pub img: String,
    pub language: String,
    // В Go: `// Date: p.Date` — закомментировано намеренно, всегда 0.
    pub date: i32,
    pub likes: [i32; 2],
    #[serde(rename = "forumId")]
    pub forum_id: i32,
    #[serde(rename = "mainWiki")]
    pub main_wiki: i32,
}

#[derive(Serialize)]
pub struct WikiProfile {
    #[serde(rename = "ID")]
    pub id: i32,
    pub title: String,
    pub text: String,
    #[serde(rename = "ban")]
    pub img: String,
    pub language: String,
    // В Go: `// Date: p.Date` — закомментировано намеренно, всегда 0.
    pub date: i32,
    #[serde(rename = "userId")]
    pub user_id: i32,
    #[serde(rename = "connGdps")]
    pub connected_gdps: i32,
    #[serde(rename = "forumId")]
    pub forum_id: i32,
    #[serde(rename = "mainWiki")]
    pub main_wiki: i32,
    #[serde(rename = "color")]
    pub colors: String,
}

#[derive(Serialize)]
pub struct WikiLT2 {
    #[serde(rename = "ID")]
    pub id: i32,
    pub title: String,
    pub text: String,
    #[serde(rename = "ban")]
    pub img: String,
    pub language: String,
    // В LT2 — единственный рендер, где date реально присваивается.
    pub date: i32,
    pub likes: [i32; 2],
    #[serde(rename = "userId")]
    pub user_id: i32,
    #[serde(rename = "connGdps")]
    pub connected_gdps: i32,
    #[serde(rename = "forumId")]
    pub forum_id: i32,
    #[serde(rename = "mainWiki")]
    pub main_wiki: i32,
    #[serde(rename = "color")]
    pub colors: String,
}

impl Wiki {
    /// Порт Wiki.ToShort(). Date намеренно 0 — см. шапку wikis.go.
    pub fn to_short(&self) -> WikiShort {
        WikiShort {
            id: self.id,
            title: self.title.clone(),
            text: self.text.clone(),
            img: self.img.clone(),
            language: self.language.clone(),
            date: 0,
            likes: [self.likes, self.disls],
            forum_id: self.forum_id,
            main_wiki: self.main_wiki,
        }
    }

    /// Порт Wiki.ToFull(). Date намеренно 0 — см. шапку wikis.go.
    pub fn to_full(&self) -> WikiProfile {
        WikiProfile {
            id: self.id,
            title: self.title.clone(),
            text: self.text.clone(),
            img: self.img.clone(),
            language: self.language.clone(),
            date: 0,
            user_id: self.user_id,
            connected_gdps: self.connected_gdps,
            forum_id: self.forum_id,
            main_wiki: self.main_wiki,
            colors: self.colors.clone(),
        }
    }

    /// Порт Wiki.ToLT2(). Здесь date — настоящее значение.
    pub fn to_lt2(&self) -> WikiLT2 {
        WikiLT2 {
            id: self.id,
            title: self.title.clone(),
            text: self.text.clone(),
            img: self.img.clone(),
            language: self.language.clone(),
            date: self.date,
            likes: [self.likes, self.disls],
            user_id: self.user_id,
            connected_gdps: self.connected_gdps,
            forum_id: self.forum_id,
            main_wiki: self.main_wiki,
            colors: self.colors.clone(),
        }
    }
}
