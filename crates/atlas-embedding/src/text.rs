pub(crate) fn prefixed_text(text: &str, prefix: &str) -> String {
    if prefix.is_empty() {
        text.to_string()
    } else {
        format!("{prefix}{text}")
    }
}

pub fn normalize_embedding_text(value: &str) -> String {
    let lower = value.to_lowercase().replace("&nbsp;", " ");
    let mut normalized = String::with_capacity(lower.len());
    let mut previous_space = true;
    for character in lower.chars() {
        if character.is_ascii_lowercase() || character.is_ascii_digit() {
            normalized.push(character);
            previous_space = false;
        } else if !previous_space {
            normalized.push(' ');
            previous_space = true;
        }
    }
    normalized.trim().to_string()
}
