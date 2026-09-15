use serde::Serialize;
use super::db::User;

#[derive(Serialize)]
pub struct UserResponse {
    #[serde(rename = "ID")]
    pub id: i32,
    pub username: String,
    #[serde(rename = "isActive")]
    pub is_active: i32,
    pub role: i32,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub token: String,
    pub resume: String,
    pub socials: String,
    #[serde(rename = "cityData")]
    pub city_data: (String, String),
}

#[derive(Serialize)]
pub struct UserPublic {
    #[serde(rename = "ID")]
    pub id: i32,
    pub username: String,
    #[serde(rename = "isActive")]
    pub is_active: i32,
    pub role: i32,
    pub resume: String,
    pub socials: String,
}

impl User {
    pub fn private_profile(&self, city: &(String, String), render_token: bool) -> UserResponse {
        let display_name = if self.nickname.is_empty() {
            self.username.clone()
        } else {
            self.nickname.clone()
        };
        let token = if render_token {
            self.token.clone()
        } else {
            String::new()
        };
        UserResponse {
            id: self.id,
            username: display_name,
            is_active: self.activated,
            role: self.priority,
            token,
            resume: self.resume.clone(),
            socials: self.socials.clone(),
            city_data: city.clone(),
        }
    }

    pub fn public_profile(&self) -> UserPublic {
        let display_name = if self.nickname.is_empty() {
            self.username.clone()
        } else {
            self.nickname.clone()
        };
        UserPublic {
            id: self.id,
            username: display_name,
            is_active: self.activated,
            role: self.priority,
            resume: self.resume.clone(),
            socials: self.socials.clone(),
        }
    }
}
