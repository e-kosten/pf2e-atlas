pub(crate) fn normalize_text(value: &str) -> String {
    atlas_domain::normalize_record_name(value)
}
