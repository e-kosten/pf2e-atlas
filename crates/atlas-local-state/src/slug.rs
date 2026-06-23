use crate::{LocalStateError, LocalStateResult};

pub fn derive_slug(name: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;
    for character in name.trim().to_lowercase().chars() {
        let next = if character.is_ascii_alphanumeric() {
            Some(character)
        } else if character.is_whitespace() || character == '-' || character == '_' {
            Some('-')
        } else {
            None
        };
        let Some(next) = next else {
            continue;
        };
        if next == '-' {
            if slug.is_empty() || previous_dash {
                continue;
            }
            previous_dash = true;
        } else {
            previous_dash = false;
        }
        slug.push(next);
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        "untitled".to_string()
    } else {
        slug
    }
}

pub(crate) fn validate_slug(slug: &str) -> LocalStateResult<()> {
    if slug.is_empty() {
        return Err(LocalStateError::InvalidSlug {
            slug: slug.to_string(),
            reason: "slug must not be empty",
        });
    }
    if slug.starts_with('-') || slug.ends_with('-') {
        return Err(LocalStateError::InvalidSlug {
            slug: slug.to_string(),
            reason: "slug must not start or end with '-'",
        });
    }
    let mut previous_dash = false;
    for byte in slug.bytes() {
        let valid = byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-';
        if !valid {
            return Err(LocalStateError::InvalidSlug {
                slug: slug.to_string(),
                reason: "use lowercase ASCII letters, digits, and '-'",
            });
        }
        if byte == b'-' && previous_dash {
            return Err(LocalStateError::InvalidSlug {
                slug: slug.to_string(),
                reason: "slug must not contain consecutive '-'",
            });
        }
        previous_dash = byte == b'-';
    }
    Ok(())
}
