pub const LANG_LIST: &[&str] = &["RU", "EN", "UA"];

pub fn is_valid_lang(lang: &str) -> bool {
    LANG_LIST.contains(&lang)
}
