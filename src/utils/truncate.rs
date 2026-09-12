use std::collections::BTreeMap;

pub fn truncate_runes(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}

/// Каноничный предел — 121 символ. Историческая деталь протокола: клиент ещё
/// с эпохи GDPS Helper 1.X проверяет по 121-му символу, нужно ли доставлять
/// ":" вместо самого 121-го символа. НЕ произвольное число — не менять
/// не проверив клиентский код.
pub const PREVIEW_TRUNCATE_LIMIT: usize = 121;

/// Обрезка превью текста, learn-aware про multilang JSON-словарь.
/// Триггер обрезки — по байтам (as in Go: len(text) > limit), сама обрезка —
/// по рунам. Сохранено 1:1 с оригинальным openGo поведением.
pub fn truncate_for_preview(text: &str, max_chars: usize) -> String {
    if let Ok(multi_map) = serde_json::from_str::<BTreeMap<String, String>>(text) {
        if !multi_map.is_empty() {
            let truncated: BTreeMap<String, String> = multi_map
                .into_iter()
                .map(|(lang, t)| (lang, truncate_runes(&t, max_chars)))
                .collect();
            return serde_json::to_string(&truncated).unwrap_or_default();
        }
    }
    truncate_runes(text, max_chars)
}
