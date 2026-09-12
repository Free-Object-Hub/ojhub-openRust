use std::collections::BTreeMap;
use super::langs::is_valid_lang;
use super::exploit_patch::exploit_patch;

#[derive(Debug, thiserror::Error)]
pub enum ParseMultiFieldError {
    #[error("malformed field entry: no tab separator")]
    NoSeparator,
    #[error("unknown language: {0}")]
    UnknownLang(String),
    #[error("duplicate language: {0}")]
    DuplicateLang(String),
    #[error("field too long for lang: {0}")]
    TooLong(String),
}

/// Парсит `raw` (записи вида "lang\ttext") в JSON-словарь.
/// BTreeMap — не HashMap: Go's json.Marshal сортирует ключи map по алфавиту,
/// нужно побайтово совпадать при сериализации.
pub fn parse_multi_field(
    raw: &[String],
    max_len: usize,
) -> Result<BTreeMap<String, String>, ParseMultiFieldError> {
    let mut result = BTreeMap::new();
    for entry in raw {
        let idx = entry.find('\t').ok_or(ParseMultiFieldError::NoSeparator)?;
        let lang = exploit_patch(&entry[..idx]);
        let text = exploit_patch(&entry[idx + 1..]);

        if !is_valid_lang(&lang) {
            return Err(ParseMultiFieldError::UnknownLang(lang));
        }
        if result.contains_key(&lang) {
            return Err(ParseMultiFieldError::DuplicateLang(lang));
        }
        if text.len() > max_len {
            return Err(ParseMultiFieldError::TooLong(lang));
        }
        if text.is_empty() {
            continue;
        }
        result.insert(lang, text);
    }
    Ok(result)
}
