use unicode_normalization::UnicodeNormalization;

/// Canonical exact-name and verified-alias lookup, without fuzzy equivalence.
pub fn normalize_record_name(value: &str) -> String {
    let normalized: String = value.nfc().flat_map(char::to_lowercase).collect();
    normalized
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .nfc()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::normalize_record_name;

    #[test]
    fn canonical_unicode_preserves_accents_and_punctuation() {
        assert_eq!(
            normalize_record_name("  ÉCOLE\u{2003}Drake \n"),
            "école drake"
        );
        assert_eq!(normalize_record_name("E\u{301}cole Drake"), "école drake");
        assert_ne!(
            normalize_record_name("Ecole Drake"),
            normalize_record_name("École Drake")
        );
        assert_ne!(
            normalize_record_name("Drake's"),
            normalize_record_name("Drakes")
        );
        assert_ne!(normalize_record_name("Ａ"), normalize_record_name("A"));
        assert_eq!(normalize_record_name("İ"), "i\u{307}");
    }
}
